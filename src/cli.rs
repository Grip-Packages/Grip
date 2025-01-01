use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about = "GitHub Release Package Manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Install a package
    Install {
        /// Package name to install
        package: String,
        /// Specific version to install
        #[arg(short, long)]
        version: Option<String>,
        /// Specific asset to install
        #[arg(short, long)]
        asset: Option<String>,
    },
    /// Initialize grip in current directory
    Init,
    /// Manage registries
    Registry {
        #[command(subcommand)]
        cmd: RegistryCommands,
    },
    /// List installed packages
    List,
}

#[derive(Debug, Subcommand)]
pub enum RegistryCommands {
    /// Add a new registry
    Add {
        /// Registry name
        name: String,
        /// Registry URL (github.com/owner/repo)
        url: String,
        /// Priority (higher numbers are checked first)
        #[arg(short, long)]
        priority: Option<i32>,
    },
    /// Remove a registry
    Remove {
        /// Registry name
        name: String,
    },
    /// List configured registries
    List,
}