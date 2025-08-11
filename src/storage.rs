use anyhow::{Context, Result};
use async_trait::async_trait;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use std::path::Path;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::BufReader;

use crate::config::Config;

#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    async fn upload_file(&self, key: &str, file_path: &Path, content_type: Option<&str>) -> Result<()>;
    async fn file_exists(&self, key: &str) -> Result<bool>;
}

pub struct S3Storage {
    bucket: Bucket,
}

impl S3Storage {
    pub async fn new(config: &Config) -> Result<Self> {
        let credentials = Credentials::new(
            Some(config.get_access_key()),
            Some(config.get_secret_key()),
            None,
            None,
            None,
        )?;

        let region = if let Some(endpoint) = config.get_s3_endpoint() {
            Region::Custom {
                region: config.get_aws_region().to_string(),
                endpoint: endpoint.to_string(),
            }
        } else {
            config.get_aws_region().parse().with_context(|| {
                format!("Invalid AWS region: {}", config.get_aws_region())
            })?
        };

        let mut bucket = Bucket::new(config.get_bucket_name(), region, credentials)
            .with_context(|| "Failed to create S3 bucket client")?
            .with_path_style(); // Use path-style URLs for better compatibility
        Ok(S3Storage { bucket: *bucket })
    }
}

#[async_trait]
impl StorageBackend for S3Storage {
    async fn upload_file(&self, key: &str, file_path: &Path, content_type: Option<&str>) -> Result<()> {
        let file = File::open(file_path).await
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;

        // Get file size for progress display
        let file_size = file.metadata().await
            .with_context(|| format!("Failed to get file metadata: {}", file_path.display()))?
            .len();

        let content_type = content_type.unwrap_or("application/octet-stream");

        println!("Uploading {} ({:.2} MB)...", file_path.display(), file_size as f64 / 1024.0 / 1024.0);

        // Use streaming upload for all files - the rust-s3 library handles multipart uploads internally
        let mut reader = BufReader::new(file);
        
        let _response = self.bucket
            .put_object_stream_with_content_type(&mut reader, key, content_type)
            .await
            .with_context(|| {
                format!(
                    "Failed to upload file to S3: {}. Please check your AWS credentials and S3 configuration. \
                    Ensure ACCESS_KEY, SECRET_KEY, BUCKET_NAME, and AWS_REGION are properly set.",
                    key
                )
            })?;

        // The rust-s3 library will return an error for non-200 status codes, so we don't need to check it explicitly
        println!("✓ Uploaded: {} -> s3://{}/{}", file_path.display(), self.bucket.name(), key);
        Ok(())
    }

    async fn file_exists(&self, key: &str) -> Result<bool> {
        match self.bucket.head_object(key).await {
            Ok((_, status_code)) => Ok(status_code == 200),
            Err(s3::error::S3Error::HttpFailWithBody(status, _)) if status == 404 => Ok(false),
            Err(_) => Err(anyhow::anyhow!("Failed to check if file exists")),
        }
    }
}