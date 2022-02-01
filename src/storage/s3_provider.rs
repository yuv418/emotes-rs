use anyhow::{Context, Result};
use s3::{bucket::Bucket, creds::Credentials};
use uuid::Uuid;

use crate::storage::StorageProvider;

pub struct S3StorageProvider {
    bucket: Bucket,
}

impl S3StorageProvider {
    pub fn new(bucket_name: String, creds: Credentials, region: String) -> Result<Self> {
        Ok(Self {
            bucket: Bucket::new(&bucket_name, region.parse()?, creds)
                .with_context(|| "Failed to open S3 bucket")?,
        })
    }
}

impl StorageProvider for S3StorageProvider {
    fn save(&self, uuid: Uuid, data: &[u8]) -> Result<()> {
        self.bucket.put_object_blocking(format!("{}", uuid), data)?;
        Ok(())
    }
    fn load(&self, uuid: Uuid) -> Result<Vec<u8>> {
        // We're ignoring the "code" value. TODO don't ignore the code value.
        Ok(self.bucket.get_object_blocking(format!("{}", uuid))?.0)
    }
}
