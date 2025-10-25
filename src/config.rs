use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub aws_region: String,
    pub s3_endpoint: Option<String>,
    overrides: HashMap<String, String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let access_key =
            env::var("S3_ACCESS_KEY").context("S3_ACCESS_KEY environment variable is required")?;
        let secret_key =
            env::var("S3_SECRET_KEY").context("S3_SECRET_KEY environment variable is required")?;
        let bucket_name = env::var("S3_BUCKET_NAME")
            .context("S3_BUCKET_NAME environment variable is required")?;
        let aws_region = env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let s3_endpoint = env::var("S3_ENDPOINT").ok();

        Ok(Config {
            access_key,
            secret_key,
            bucket_name,
            aws_region,
            s3_endpoint,
            overrides: HashMap::new(),
        })
    }

    pub fn set_override(&mut self, key: &str, value: &str) {
        self.overrides.insert(key.to_string(), value.to_string());
    }

    pub fn get_access_key(&self) -> &str {
        self.overrides
            .get("ACCESS_KEY")
            .map(|s| s.as_str())
            .unwrap_or(&self.access_key)
    }

    pub fn get_secret_key(&self) -> &str {
        self.overrides
            .get("SECRET_KEY")
            .map(|s| s.as_str())
            .unwrap_or(&self.secret_key)
    }

    pub fn get_bucket_name(&self) -> &str {
        self.overrides
            .get("BUCKET_NAME")
            .map(|s| s.as_str())
            .unwrap_or(&self.bucket_name)
    }

    pub fn get_aws_region(&self) -> &str {
        self.overrides
            .get("AWS_REGION")
            .map(|s| s.as_str())
            .unwrap_or(&self.aws_region)
    }

    pub fn get_s3_endpoint(&self) -> Option<&str> {
        self.overrides
            .get("S3_ENDPOINT")
            .map(|s| s.as_str())
            .or(self.s3_endpoint.as_deref())
    }
}
