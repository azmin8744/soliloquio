use super::StorageError;
use std::path::{Component, PathBuf};

pub struct LocalStorageDriver {
    pub base_dir: PathBuf,
}

fn safe_join(base: &PathBuf, key: &str) -> Result<PathBuf, StorageError> {
    for component in std::path::Path::new(key).components() {
        match component {
            Component::Normal(_) => {}
            _ => return Err(StorageError(format!("invalid key: {key}"))),
        }
    }
    Ok(base.join(key))
}

impl LocalStorageDriver {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self { base_dir: base_dir.into() }
    }

    pub fn put(&self, key: &str, data: Vec<u8>) -> Result<(), StorageError> {
        let path = safe_join(&self.base_dir, key)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| StorageError(format!("create_dir_all: {e}")))?;
        }
        std::fs::write(&path, data)
            .map_err(|e| StorageError(format!("write {key}: {e}")))
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let path = safe_join(&self.base_dir, key)?;
        std::fs::read(&path).map_err(|e| StorageError(format!("read {key}: {e}")))
    }

    pub fn delete_dir(&self, prefix: &str) -> Result<(), StorageError> {
        let path = safe_join(&self.base_dir, prefix)?;
        if path.exists() {
            std::fs::remove_dir_all(&path)
                .map_err(|e| StorageError(format!("remove_dir_all {prefix}: {e}")))?;
        }
        Ok(())
    }

    pub fn url(&self, key: &str) -> String {
        format!("/assets/{key}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn base() -> PathBuf {
        PathBuf::from("/uploads")
    }

    #[test]
    fn rejects_dotdot() {
        assert!(safe_join(&base(), "../etc/passwd").is_err());
    }

    #[test]
    fn rejects_dotdot_in_middle() {
        assert!(safe_join(&base(), "uuid/../etc/passwd").is_err());
    }

    #[test]
    fn rejects_dotdot_at_end() {
        assert!(safe_join(&base(), "uuid/..").is_err());
    }

    #[test]
    fn rejects_dot_segment() {
        assert!(safe_join(&base(), "./file.webp").is_err());
    }

    #[test]
    fn rejects_absolute_path() {
        assert!(safe_join(&base(), "/etc/passwd").is_err());
    }

    #[test]
    fn allows_normal_key() {
        let result = safe_join(&base(), "550e8400-e29b-41d4-a716-446655440000/thumbnail.webp");
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with(&base()));
    }

    #[test]
    fn allows_single_segment() {
        let result = safe_join(&base(), "thumbnail.webp");
        assert!(result.is_ok());
    }

    #[test]
    fn get_rejects_traversal() {
        let driver = LocalStorageDriver::new("/uploads");
        assert!(driver.get("../etc/passwd").is_err());
    }

    #[test]
    fn put_rejects_traversal() {
        let driver = LocalStorageDriver::new("/uploads");
        assert!(driver.put("../evil.webp", vec![]).is_err());
    }

    #[test]
    fn delete_dir_rejects_traversal() {
        let driver = LocalStorageDriver::new("/uploads");
        assert!(driver.delete_dir("../other_dir").is_err());
    }
}
