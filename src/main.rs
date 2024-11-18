mod cli;
mod config;
mod error;
mod package;
mod path;
mod registry;
mod utils;

use std::sync::Arc;

use cli::{Cli, Commands, RegistryCommands};
use colored::Colorize;
use config::Config;
use error::Result;
use lazy_static::lazy_static;
use registry::RegistryManager;
use clap::Parser;
use dialoguer::Select;

struct Grip {
    config: Config,
    registry_manager: RegistryManager,
}
lazy_static!(
    static ref CLI: Arc<Cli> = Arc::new(Cli::parse());
);

impl Grip {
    async fn new() -> Result<Self> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get local data directory"))?
            .join("grip");

        std::fs::create_dir_all(&data_dir)?;

        let config = Config::load()?;
        let registry_manager = RegistryManager::new(data_dir);

        Ok(Self {
            config,
            registry_manager,
        })
    }

    async fn install(&self, package_name: &str, version: Option<String>, asset: Option<String>) -> Result<()> {
        println!("{} Looking up package {}", "→".blue(), package_name.cyan());

        // Find package in registry
        let package = self.registry_manager.find_package(&self.config.registries, package_name).await?;
        
        println!("{} Found package in repository: {}", "→".blue(), package.info.repository.cyan());

        // Get releases from GitHub
        let releases = self.registry_manager.get_releases(&package.info.repository).await?;

        if releases.is_empty() {
            anyhow::bail!("No releases found for package '{}'", package_name);
        }

        // Select release version
        let release = match version {
            Some(ref v) => releases
                .iter()
                .find(|r| r["tag_name"].as_str().unwrap_or("") == v)
                .ok_or_else(|| anyhow::anyhow!("Version {} not found", v))?,
            None => {
                let versions: Vec<&str> = releases
                    .iter()
                    .filter_map(|r| r["tag_name"].as_str())
                    .collect();

                println!("{} Available versions:", "→".blue());
                let selection = Select::new()
                    .with_prompt("Select version")
                    .items(&versions)
                    .default(0)
                    .interact()?;

                &releases[selection]
            }
        };

        // Select asset
        let assets = release["assets"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No assets found in release"))?;

        let asset_obj = match asset {
            Some(ref a) => assets
                .iter()
                .find(|asset| asset["name"].as_str().unwrap_or("") == a)
                .ok_or_else(|| anyhow::anyhow!("Asset {} not found", a))?,
            None => {
                let asset_names: Vec<&str> = assets
                    .iter()
                    .filter_map(|a| a["name"].as_str())
                    .collect();

                println!("{} Available assets:", "→".blue());
                let selection = Select::new()
                    .with_prompt("Select asset")
                    .items(&asset_names)
                    .default(0)
                    .interact()?;

                &assets[selection]
            }
        };

        let download_url = asset_obj["browser_download_url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid download URL"))?;
        
        let filename = asset_obj["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid asset name"))?;

        // Download and install
        let target_dir = self.registry_manager.data_dir
            .join("packages")
            .join(package_name)
            .join(release["tag_name"].as_str().unwrap_or("unknown"));

        let downloaded_file = self.registry_manager
            .download_asset(download_url, filename, &target_dir)
            .await?;

        // Handle archive extraction if needed
        if filename.ends_with(".zip") || filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
            println!("{} Extracting archive...", "→".blue());
            utils::extract_archive(&downloaded_file, &target_dir);
            println!("{} Extracted to {:?}", "✓".green(), target_dir);
            
            // Clean up archive after extraction
            std::fs::remove_file(downloaded_file)?;
        }

        // Add to PATH if needed
        path::add_to_path(&target_dir).await?;

        println!("{} Installation complete!", "✓".green());
        Ok(())
    }

    async fn handle_registry_command(&mut self, cmd: RegistryCommands) -> Result<()> {
        match cmd {
            RegistryCommands::Add { name, url, priority } => {
                if self.config.registries.iter().any(|r| r.name == name) {
                    anyhow::bail!("Registry '{}' already exists", name);
                }

                self.config.registries.push(config::Registry {
                    name: name.clone(),
                    url: url.clone(),
                    priority: priority.unwrap_or(0),
                });

                self.config.save()?;
                println!("{} Added registry {} ({})", "✓".green(), name.cyan(), url);
            }
            RegistryCommands::Remove { name } => {
                if name == "default" {
                    anyhow::bail!("Cannot remove default registry");
                }

                let original_len = self.config.registries.len();
                self.config.registries.retain(|r| r.name != name);

                if self.config.registries.len() == original_len {
                    anyhow::bail!("Registry '{}' not found", name);
                }

                self.config.save()?;

                // Remove cached registry
                let registry_path = self.registry_manager.data_dir
                    .join("registries")
                    .join(&name);
                if registry_path.exists() {
                    std::fs::remove_dir_all(registry_path)?;
                }

                println!("{} Removed registry {}", "✓".green(), name.cyan());
            }
            RegistryCommands::List => {
                println!("{} Configured registries:", "→".blue());
                for registry in &self.config.registries {
                    println!("  {} {} (priority: {}, url: {})",
                        "→".blue(),
                        registry.name.cyan(),
                        registry.priority,
                        registry.url
                    );
                }
            }
        }
        Ok(())
    }

    async fn init(&self) -> Result<()> {
        
        let config = serde_json::json!({
            "name": "grip-project",
            "version": "0.1.0",
            "dependencies": {}
        });

        std::fs::write(
            "grip.json",
            serde_json::to_string_pretty(&config)?,
        )?;

        println!("{} Created grip.json", "✓".green());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {

    let cli = Arc::clone(&CLI);


    print_debug!("dasdfasfd");
    let mut grip = Grip::new().await?;
    let command = cli.command.clone();
    match command {
        Commands::Install { package, version, asset } => {
            grip.install(&package, version, asset).await?;
        }
        Commands::Registry { cmd } => {
            grip.handle_registry_command(cmd).await?;
        }
        Commands::Init => {
            grip.init().await?;
        }
    }

    Ok(())
}