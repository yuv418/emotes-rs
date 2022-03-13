use anyhow::{Context, Result};
use s3::{bucket::Bucket, creds::Credentials};
use serde::Deserialize;
use uuid::Uuid;

use crate::storage::StorageProvider;

pub struct S3StorageProvider {
    bucket: Bucket,
}

impl S3StorageProvider {
    pub fn new(config: &S3StorageProviderConfig) -> Result<Self> {
        Ok(Self {
            bucket: Bucket::new(
                &config.bucket,
                config.region.parse()?,
                Credentials::new(
                    config.credentials.access_key.as_deref(),
                    config.credentials.secret_key.as_deref(),
                    config.credentials.security_token.as_deref(),
                    config.credentials.session_token.as_deref(),
                    config.credentials.profile.as_deref(),
                )?,
            )
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
    fn delete(&self, uuid: uuid::Uuid) -> Result<()> {
        self.bucket.delete_object_blocking(format!("{}", uuid))?;
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct S3CredentialsConfig {
    access_key: Option<String>,
    secret_key: Option<String>,
    security_token: Option<String>,
    session_token: Option<String>,
    profile: Option<String>,
}

#[derive(Deserialize)]
pub struct S3StorageProviderConfig {
    bucket: String,
    region: String,
    credentials: crate::storage::s3_provider::S3CredentialsConfig,
}
