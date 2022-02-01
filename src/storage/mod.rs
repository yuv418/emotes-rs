use anyhow::Result;
use uuid::Uuid;

pub trait StorageProvider {
    fn save(&self, uuid: Uuid, ext: String, data: &[u8]) -> Result<()>;
    fn load(&self, uuid: Uuid, ext: String) -> Result<Vec<u8>>;
}

mod local_provider;
mod s3_provider;

pub use local_provider::LocalStorageProvider;
pub use s3_provider::S3StorageProvider;
