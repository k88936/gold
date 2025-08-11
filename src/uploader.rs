use anyhow::{Context, Result};
use glob::glob;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::storage::StorageBackend;

pub struct ReleaseUploader {
    storage: Box<dyn StorageBackend>,
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub file_path: PathBuf,
    pub display_name: Option<String>,
    pub s3_key: String,
}

impl ReleaseUploader {
    pub fn new(storage: Box<dyn StorageBackend>) -> Self {
        Self { storage }
    }

    pub async fn upload_release(
        &self,
        package_name: &str,
        tag: &str,
        file_patterns: &[String],
    ) -> Result<()> {
        let assets = self.discover_assets(package_name, tag, file_patterns)?;

        if assets.is_empty() {
            anyhow::bail!("No assets found matching the specified patterns");
        }

        println!("Found {} assets to upload:", assets.len());
        for asset in &assets {
            println!("  {} -> {}", asset.file_path.display(), asset.s3_key);
        }

        for asset in assets {
            let content_type = guess_content_type(&asset.file_path);
            self.storage
                .upload_file(&asset.s3_key, &asset.file_path, content_type)
                .await
                .with_context(|| {
                    format!("Failed to upload asset: {}", asset.file_path.display())
                })?;
        }

        Ok(())
    }

    fn discover_assets(
        &self,
        package_name: &str,
        tag: &str,
        file_patterns: &[String],
    ) -> Result<Vec<Asset>> {
        let mut assets = Vec::new();
        let mut seen_keys = HashSet::new();

        for pattern in file_patterns {
            let (file_pattern, display_name) = self.parse_pattern(pattern);
            let matched_files = self.find_matching_files(&file_pattern)?;

            for file_path in matched_files {
                let s3_key = self.generate_s3_key(package_name, tag, &file_path);

                // Avoid duplicate keys
                if seen_keys.contains(&s3_key) {
                    continue;
                }
                seen_keys.insert(s3_key.clone());

                assets.push(Asset {
                    file_path,
                    display_name: display_name.clone(),
                    s3_key,
                });
            }
        }

        Ok(assets)
    }

    fn parse_pattern(&self, pattern: &str) -> (String, Option<String>) {
        if let Some(hash_pos) = pattern.find('#') {
            let file_pattern = pattern[..hash_pos].trim().to_string();
            let display_name = pattern[hash_pos + 1..].trim().to_string();
            (file_pattern, Some(display_name))
        } else {
            (pattern.to_string(), None)
        }
    }

    fn find_matching_files(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let path = Path::new(pattern);

        // If it's a direct file path and exists, return it
        if path.is_file() {
            return Ok(vec![path.to_path_buf()]);
        }

        // If it's a directory, reject it - user should use glob patterns like "dir/*"
        if path.is_dir() {
            anyhow::bail!(
                "Directory '{}' cannot be uploaded directly. Use a glob pattern like '{}/*' to upload files from the directory.",
                path.display(),
                path.display()
            );
        }

        // Try to match as a glob-like pattern
        self.match_glob_pattern(pattern)
    }

    fn match_glob_pattern(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        // Use the glob crate for proper glob pattern matching
        match glob(pattern) {
            Ok(paths) => {
                for entry in paths {
                    match entry {
                        Ok(path) => {
                            if path.is_file() {
                                files.push(path);
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Error reading path in glob pattern: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                anyhow::bail!("Invalid glob pattern '{}': {}", pattern, e);
            }
        }

        Ok(files)
    }

    fn generate_s3_key(&self, package_name: &str, tag: &str, file_path: &Path) -> String {
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        format!("{}/{}/{}", package_name, tag, filename)
    }
}

fn guess_content_type(file_path: &Path) -> Option<&'static str> {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("zip") => Some("application/zip"),
        Some("tar") => Some("application/x-tar"),
        Some("gz") => Some("application/gzip"),
        Some("exe") => Some("application/octet-stream"),
        Some("msi") => Some("application/x-msi"),
        Some("dmg") => Some("application/x-apple-diskimage"),
        Some("deb") => Some("application/vnd.debian.binary-package"),
        Some("rpm") => Some("application/x-rpm"),
        Some("json") => Some("application/json"),
        Some("txt") => Some("text/plain"),
        Some("md") => Some("text/markdown"),
        _ => Some("application/octet-stream"),
    }
}
