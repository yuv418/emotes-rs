use crate::config::{EmotesConfigStorageProvider, EMOTES_CONFIG};
use anyhow::Result;
use lazy_static::lazy_static;
use uuid::Uuid;

pub trait StorageProvider {
    fn save(&self, uuid: Uuid, data: &[u8]) -> Result<()>;
    fn load(&self, uuid: Uuid) -> Result<Vec<u8>>;
    fn delete(&self, uuid: Uuid) -> Result<()>;
}

mod local_provider;
mod s3_provider;

pub use local_provider::{LocalStorageProvider, LocalStorageProviderConfig};
pub use s3_provider::{S3StorageProvider, S3StorageProviderConfig};

// TODO make STORAGE_PROVIDER dynamically configurable from EMOTES_CONFIG
lazy_static! {
    pub static ref STORAGE_PROVIDER: Box<dyn StorageProvider + Sync> =
        match &EMOTES_CONFIG.storage_provider {
            EmotesConfigStorageProvider::Local(config) =>
                Box::new(LocalStorageProvider::new(config).unwrap()),
            EmotesConfigStorageProvider::S3(config) =>
                Box::new(S3StorageProvider::new(config).unwrap()),
        };
}
