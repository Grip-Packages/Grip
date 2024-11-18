use crate::error::{Result, GripError};
use crate::package::Package;
use crate::config::Registry;
use colored::Colorize;
use tokio::process::Command;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use std::path::PathBuf;

pub struct RegistryManager {
    pub data_dir: PathBuf,
    client: reqwest::Client,
}

impl RegistryManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            client: reqwest::Client::new(),
        }
    }

    pub async fn ensure_registry(&self, registry: &Registry) -> Result<PathBuf> {
        let registry_path = self.data_dir.join("registries").join(&registry.name);
        
        if !registry_path.exists() {
            println!("{} Cloning registry {}...", "→".blue(), registry.name.cyan());
            let url = format!("https://{}.git", registry.url);
            let status = Command::new("git")
                .args(["clone", "--depth", "1", &url, &registry_path.to_string_lossy()])
                .status()
                .await?;

            if !status.success() {
                return Err(GripError::RegistryError(format!(
                    "Failed to clone registry {}", 
                    registry.name
                )).into());
            }
        } else {
            println!("{} Updating registry {}...", "→".blue(), registry.name.cyan());
            let status = Command::new("git")
                .args(["pull", "--ff-only"])
                .current_dir(&registry_path)
                .status()
                .await?;

            if !status.success() {
                return Err(GripError::RegistryError(format!(
                    "Failed to update registry {}", 
                    registry.name
                )).into());
            }
        }

        Ok(registry_path)
    }

    pub async fn find_package(&self, registries: &[Registry], package_name: &str) -> Result<Package> {
        // Sort registries by priority (highest first)
        let mut sorted_registries = registries.to_vec();
        sorted_registries.sort_by(|a, b| b.priority.cmp(&a.priority));

        for registry in sorted_registries {
            let registry_path = self.ensure_registry(&registry).await?;
            let packages_path = registry_path.join("packages");
            
            if packages_path.exists() {
                let package_file = packages_path.join(format!("{}.json", package_name));
                if package_file.exists() {
                    return Package::load(package_file);
                }
            }
        }

        Err(GripError::PackageNotFound(package_name.to_string()).into())
    }

    pub async fn get_releases(&self, repo: &str) -> Result<Vec<serde_json::Value>> {
        let releases_url = format!(
            "https://api.github.com/repos/{}/releases",
            repo
        );

        let response = self.client
            .get(&releases_url)
            .header("User-Agent", "grip")
            .send()
            .await?;

        if !response.status().is_success() {
            if response.status() == 404 {
                return Err(GripError::RegistryError(format!(
                    "Repository '{}' not found on GitHub", 
                    repo
                )).into());
            }
            let status = response.status();
            let error_text = response.text().await?;
            return Err(GripError::RegistryError(format!(
                "GitHub API error ({}): {}", 
                status, error_text
            )).into());
        }

        let releases = response
            .json()
            .await
            .map_err(|e| GripError::RegistryError(format!(
                "Failed to parse GitHub releases: {}", 
                e
            )))?;

        Ok(releases)
    }

    pub async fn download_asset(
        &self, 
        url: &str, 
        filename: &str,
        target_dir: &PathBuf
    ) -> Result<PathBuf> {
        println!("{} Downloading {}", "→".blue(), filename.cyan());

        let response = self.client
            .get(url)
            .send()
            .await?;

        let total_size = response.content_length().unwrap_or(0);
        
        let pb = indicatif::ProgressBar::new(total_size);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-")
        );

        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path().join(filename);
        let mut file = tokio::fs::File::create(&temp_path).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Download complete!");

        // Create target directory if it doesn't exist
        if !target_dir.exists() {
            tokio::fs::create_dir_all(target_dir).await?;
        }

        let final_path = target_dir.join(filename);
        tokio::fs::copy(&temp_path, &final_path).await?;
        tokio::fs::remove_file(&temp_path).await?;

        Ok(final_path)
    }
}