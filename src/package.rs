use serde::{Deserialize, Serialize};
use crate::error::Result;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub repository: String,
    pub description: Option<String>,
    pub executable_name: Option<String>,
}

pub struct Package {
    pub info: PackageInfo,
    pub path: PathBuf,
}

impl Package {
    pub fn load(path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let info: PackageInfo = serde_json::from_str(&content)?;
        Ok(Self { info, path })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}