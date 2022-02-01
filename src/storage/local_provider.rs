use crate::storage::StorageProvider;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub struct LocalStorageProvider {
    base_path: PathBuf,
}

impl LocalStorageProvider {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl StorageProvider for LocalStorageProvider {
    fn save(&self, uuid: Uuid, ext: String, data: &[u8]) -> Result<()> {
        fs::write(self.base_path.join(format!("{}/{}", uuid, ext)), data)?;
        Ok(())
    }
    fn load(&self, uuid: Uuid, ext: String) -> Result<Vec<u8>> {
        Ok(fs::read(self.base_path.join(format!("{}/{}", uuid, ext)))?)
    }
}
