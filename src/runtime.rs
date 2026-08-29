use crate::domain::{CHROMIUM_COMPATIBILITY, PROFILE_SCHEMA_VERSION, Profile};
use serde::Serialize;
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Stdio,
};
use thiserror::Error;
use tokio::{
    process::{Child, Command},
    sync::Mutex,
};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("profile {0} is already running")]
    AlreadyRunning(Uuid),
    #[error("browser compatibility mismatch: expected {expected}, got {actual}")]
    Incompatible {
        expected: &'static str,
        actual: String,
    },
    #[error("browser runtime failed: {0}")]
    Io(#[from] io::Error),
    #[error("snapshot serialization failed: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Serialize)]
struct RuntimeSnapshot<'a> {
    schema_version: u32,
    chromium_compatibility: &'static str,
    profile: &'a Profile,
}

struct Running {
    profile_id: Uuid,
    child: Child,
    snapshot: PathBuf,
}

pub struct ChromiumRuntime {
    executable: PathBuf,
    data_root: PathBuf,
    running: Mutex<Vec<Running>>,
}

impl ChromiumRuntime {
    pub fn new(
        executable: impl Into<PathBuf>,
        data_root: impl Into<PathBuf>,
    ) -> Result<Self, RuntimeError> {
        let data_root = data_root.into();
        fs::create_dir_all(&data_root)?;
        Ok(Self {
            executable: executable.into(),
            data_root,
            running: Mutex::new(Vec::new()),
        })
    }

    pub async fn start(&self, profile: &Profile) -> Result<(), RuntimeError> {
        let mut running = self.running.lock().await;
        reap(&mut running).await;
        if running.iter().any(|v| v.profile_id == profile.id) {
            return Err(RuntimeError::AlreadyRunning(profile.id));
        }
        self.verify_build().await?;
        let profile_root = self.data_root.join(profile.id.to_string());
        fs::create_dir_all(&profile_root)?;
        let snapshot = profile_root.join(format!("runtime-{}.json", profile.revision));
        write_private(
            &snapshot,
            &serde_json::to_vec(&RuntimeSnapshot {
                schema_version: PROFILE_SCHEMA_VERSION,
                chromium_compatibility: CHROMIUM_COMPATIBILITY,
                profile,
            })?,
        )?;
        let user_data = profile_root.join("user-data");
        fs::create_dir_all(&user_data)?;
        let child = Command::new(&self.executable)
            .arg(format!(
                "--antidetect-profile-snapshot={}",
                snapshot.display()
            ))
            .arg(format!("--user-data-dir={}", user_data.display()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()?;
        running.push(Running {
            profile_id: profile.id,
            child,
            snapshot,
        });
        Ok(())
    }

    pub async fn stop(&self, id: Uuid) -> Result<bool, RuntimeError> {
        let mut running = self.running.lock().await;
        let Some(index) = running.iter().position(|v| v.profile_id == id) else {
            return Ok(false);
        };
        let mut item = running.swap_remove(index);
        item.child.kill().await?;
        let _ = fs::remove_file(item.snapshot);
        Ok(true)
    }

    async fn verify_build(&self) -> Result<(), RuntimeError> {
        let output = Command::new(&self.executable)
            .arg("--antidetect-query-build-id")
            .output()
            .await?;
        let actual = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if !output.status.success() || actual != CHROMIUM_COMPATIBILITY {
            return Err(RuntimeError::Incompatible {
                expected: CHROMIUM_COMPATIBILITY,
                actual,
            });
        }
        Ok(())
    }
}

async fn reap(items: &mut Vec<Running>) {
    let mut index = 0;
    while index < items.len() {
        if items[index].child.try_wait().ok().flatten().is_some() {
            let item = items.swap_remove(index);
            let _ = fs::remove_file(item.snapshot);
        } else {
            index += 1;
        }
    }
}

fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)
}
