use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

pub const PROFILE_SCHEMA_VERSION: u32 = 1;
pub const CHROMIUM_COMPATIBILITY: &str = "antidetection/152.0.7977.64/profile-v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub id: Uuid,
    pub revision: u64,
    pub name: String,
    pub region: RegionProfile,
    pub hardware: HardwareProfile,
    pub proxy: Option<ProxyProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegionProfile {
    pub country_code: String,
    pub timezone: String,
    pub locale: String,
    pub languages: Vec<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HardwareProfile {
    pub architecture: Architecture,
    pub cpu_class: String,
    pub logical_cores: u16,
    pub memory_gib: u16,
    pub gpu_class: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Architecture {
    X86_64,
    Arm64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProxyProfile {
    pub scheme: ProxyScheme,
    pub host: String,
    pub port: u16,
    pub resolve_dns_through_proxy: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProxyScheme {
    Http,
    Https,
    Socks5,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationIssue {
    pub path: &'static str,
    pub message: &'static str,
}

impl Profile {
    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        if self.name.trim().is_empty() {
            issues.push(issue("name", "must not be empty"));
        }
        if !is_country_code(&self.region.country_code) {
            issues.push(issue(
                "region.country_code",
                "must be a two-letter uppercase ISO code",
            ));
        }
        if !is_timezone(&self.region.timezone) {
            issues.push(issue("region.timezone", "must be an IANA timezone name"));
        }
        if !is_locale(&self.region.locale) {
            issues.push(issue("region.locale", "must be a supported BCP 47 locale"));
        }
        if self.region.languages.is_empty() || self.region.languages.iter().any(|v| !is_locale(v)) {
            issues.push(issue(
                "region.languages",
                "must contain supported BCP 47 locales",
            ));
        }
        match (self.region.latitude, self.region.longitude) {
            (Some(lat), Some(lon))
                if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon) => {}
            (None, None) => {}
            _ => issues.push(issue(
                "region",
                "latitude and longitude must both be present and in range",
            )),
        }
        if ![2, 4, 8, 12, 16, 20, 24, 32].contains(&self.hardware.logical_cores) {
            issues.push(issue(
                "hardware.logical_cores",
                "is not a supported hardware bucket",
            ));
        }
        if ![2, 4, 8, 16, 32, 64].contains(&self.hardware.memory_gib) {
            issues.push(issue(
                "hardware.memory_gib",
                "is not a supported hardware bucket",
            ));
        }
        if self.hardware.cpu_class.trim().is_empty() {
            issues.push(issue("hardware.cpu_class", "must identify a catalog class"));
        }
        if self.hardware.gpu_class.trim().is_empty() {
            issues.push(issue("hardware.gpu_class", "must identify a catalog class"));
        }
        if let Some(proxy) = &self.proxy {
            if proxy.host.trim().is_empty()
                || proxy.host.parse::<IpAddr>().is_err() && !is_hostname(&proxy.host)
            {
                issues.push(issue("proxy.host", "must be an IP address or DNS hostname"));
            }
            if matches!(proxy.scheme, ProxyScheme::Socks5) && !proxy.resolve_dns_through_proxy {
                issues.push(issue(
                    "proxy.resolve_dns_through_proxy",
                    "SOCKS5 profiles must prevent local DNS leaks",
                ));
            }
        }
        issues
    }
}

fn issue(path: &'static str, message: &'static str) -> ValidationIssue {
    ValidationIssue { path, message }
}
fn is_country_code(v: &str) -> bool {
    v.len() == 2 && v.bytes().all(|b| b.is_ascii_uppercase())
}
fn is_timezone(v: &str) -> bool {
    v.contains('/') && !v.contains([' ', '\\'])
}
fn is_locale(v: &str) -> bool {
    let mut parts = v.split('-');
    matches!(parts.next(), Some(language) if (2..=3).contains(&language.len()) && language.bytes().all(|b| b.is_ascii_lowercase()))
        && parts.all(|part| {
            (2..=8).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_alphanumeric())
        })
}
fn is_hostname(v: &str) -> bool {
    v.len() <= 253
        && v.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn valid_profile() -> Profile {
        Profile {
            id: Uuid::new_v4(),
            revision: 0,
            name: "Berlin QA".into(),
            region: RegionProfile {
                country_code: "DE".into(),
                timezone: "Europe/Berlin".into(),
                locale: "de-DE".into(),
                languages: vec!["de-DE".into(), "de".into()],
                latitude: Some(52.52),
                longitude: Some(13.405),
            },
            hardware: HardwareProfile {
                architecture: Architecture::X86_64,
                cpu_class: "intel-8c-v1".into(),
                logical_cores: 8,
                memory_gib: 16,
                gpu_class: "intel-iris-xe-v1".into(),
            },
            proxy: Some(ProxyProfile {
                scheme: ProxyScheme::Socks5,
                host: "proxy.example.com".into(),
                port: 1080,
                resolve_dns_through_proxy: true,
            }),
        }
    }

    #[test]
    fn accepts_coherent_profile() {
        assert!(valid_profile().validate().is_empty());
    }

    #[test]
    fn reports_all_invalid_fields() {
        let mut p = valid_profile();
        p.name = " ".into();
        p.region.country_code = "Germany".into();
        p.hardware.logical_cores = 7;
        p.proxy.as_mut().unwrap().resolve_dns_through_proxy = false;
        assert_eq!(p.validate().len(), 4);
    }
}
