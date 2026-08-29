use crate::domain::Profile;
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Mutex,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("profile {0} was not found")]
    NotFound(Uuid),
    #[error("profile revision conflict: expected {expected}, actual {actual}")]
    Conflict { expected: u64, actual: u64 },
    #[error("profile storage failed: {0}")]
    Io(#[from] io::Error),
    #[error("profile data is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("profile storage is unavailable after an internal synchronization failure")]
    Unavailable,
}

pub struct ProfileStore {
    root: PathBuf,
    gate: Mutex<()>,
}

impl ProfileStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        Ok(Self {
            root,
            gate: Mutex::new(()),
        })
    }
    pub fn list(&self) -> Result<Vec<Profile>, StoreError> {
        let _guard = self.gate.lock().map_err(|_| StoreError::Unavailable)?;
        let mut profiles = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.extension().is_some_and(|v| v == "json") {
                profiles.push(read(&path)?);
            }
        }
        profiles.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
        Ok(profiles)
    }
    pub fn get(&self, id: Uuid) -> Result<Profile, StoreError> {
        let _guard = self.gate.lock().map_err(|_| StoreError::Unavailable)?;
        self.get_unlocked(id)
    }
    fn get_unlocked(&self, id: Uuid) -> Result<Profile, StoreError> {
        let path = self.path(id);
        if !path.exists() {
            return Err(StoreError::NotFound(id));
        }
        read(&path)
    }
    pub fn create(&self, mut profile: Profile) -> Result<Profile, StoreError> {
        let _guard = self.gate.lock().map_err(|_| StoreError::Unavailable)?;
        let path = self.path(profile.id);
        if path.exists() {
            return Err(StoreError::Conflict {
                expected: 0,
                actual: self.get_unlocked(profile.id)?.revision,
            });
        }
        profile.revision = 1;
        write_atomic(&path, &profile)?;
        Ok(profile)
    }
    pub fn update(&self, mut profile: Profile, expected: u64) -> Result<Profile, StoreError> {
        let _guard = self.gate.lock().map_err(|_| StoreError::Unavailable)?;
        let current = self.get_unlocked(profile.id)?;
        if current.revision != expected {
            return Err(StoreError::Conflict {
                expected,
                actual: current.revision,
            });
        }
        profile.revision = current.revision + 1;
        write_atomic(&self.path(profile.id), &profile)?;
        Ok(profile)
    }
    pub fn delete(&self, id: Uuid, expected: u64) -> Result<(), StoreError> {
        let _guard = self.gate.lock().map_err(|_| StoreError::Unavailable)?;
        let current = self.get_unlocked(id)?;
        if current.revision != expected {
            return Err(StoreError::Conflict {
                expected,
                actual: current.revision,
            });
        }
        fs::remove_file(self.path(id))?;
        Ok(())
    }
    fn path(&self, id: Uuid) -> PathBuf {
        self.root.join(format!("{id}.json"))
    }
}
fn read(path: &Path) -> Result<Profile, StoreError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn write_atomic(path: &Path, value: &Profile) -> Result<(), StoreError> {
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, serde_json::to_vec_pretty(value)?)?;
    fs::rename(temp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tests::valid_profile;
    #[test]
    fn revisions_prevent_lost_updates() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProfileStore::new(dir.path()).unwrap();
        let p = store.create(valid_profile()).unwrap();
        let updated = store.update(p.clone(), 1).unwrap();
        assert_eq!(updated.revision, 2);
        assert!(matches!(
            store.update(p, 1),
            Err(StoreError::Conflict { actual: 2, .. })
        ));
    }
}
