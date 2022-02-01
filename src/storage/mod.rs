use anyhow::Result;
use uuid::Uuid;

use crate::config::EMOTES_CONFIG;
use lazy_static::lazy_static;

pub trait StorageProvider {
    fn save(&self, uuid: Uuid, data: &[u8]) -> Result<()>;
    fn load(&self, uuid: Uuid) -> Result<Vec<u8>>;
}

mod local_provider;
mod s3_provider;

pub use local_provider::LocalStorageProvider;
pub use s3_provider::S3StorageProvider;

// TODO make STORAGE_PROVIDER dynamically configurable from EMOTES_CONFIG
lazy_static! {
    pub static ref STORAGE_PROVIDER: LocalStorageProvider =
        LocalStorageProvider::new(EMOTES_CONFIG.data_dir.join("emotes"));
}
