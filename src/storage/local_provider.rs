use crate::storage::StorageProvider;
use anyhow::Result;
use log::info;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub struct LocalStorageProvider {
    base_path: PathBuf,
}

impl LocalStorageProvider {
    pub fn new(config: &LocalStorageProviderConfig) -> Result<Self> {
        if !config.data_dir.exists() {
            info!("creating local storage dir for emotes");
            fs::create_dir_all(&config.data_dir)?;
        }
        Ok(Self {
            base_path: config.data_dir.clone(),
        })
    }
}

impl StorageProvider for LocalStorageProvider {
    fn save(&self, uuid: Uuid, data: &[u8]) -> Result<()> {
        fs::write(self.base_path.join(format!("{}", uuid)), data)?;
        Ok(())
    }
    fn load(&self, uuid: Uuid) -> Result<Vec<u8>> {
        Ok(fs::read(self.base_path.join(format!("{}", uuid)))?)
    }
    fn delete(&self, uuid: Uuid) -> Result<()> {
        fs::remove_file(self.base_path.join(format!("{}", uuid)))?;
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct LocalStorageProviderConfig {
    data_dir: PathBuf,
}
