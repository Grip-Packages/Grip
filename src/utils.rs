use crate::error::Result;
use std::path::Path;
#[macro_export]
macro_rules! print_debug {
    ($($arg:tt)*) => {
        if CLI.debug {
            println!($($arg)*);
        }
    };
}
pub async fn extract_archive(archive_path: &Path, target_dir: &Path) -> Result<()> {
    if archive_path.extension().map_or(false, |ext| ext == "zip") {
        let file = std::fs::File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        archive.extract(target_dir)?;
    } else if archive_path.extension().map_or(false, |ext| ext == "gz" || ext == "tgz") {
        use std::process::Command;
        
        Command::new("tar")
            .args(&["xzf", &archive_path.to_string_lossy()])
            .current_dir(target_dir)
            .status()?;
    }
    
    Ok(())
}

pub fn get_platform() -> &'static str {
    #[cfg(target_os = "windows")]
    return "windows";
    
    #[cfg(target_os = "macos")]
    return "macos";
    
    #[cfg(target_os = "linux")]
    return "linux";
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return "unknown";
}

pub fn get_arch() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    return "x86_64";
    
    #[cfg(target_arch = "aarch64")]
    return "aarch64";
    
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    return "unknown";
}

pub fn get_binary_extension() -> &'static str {
    #[cfg(target_os = "windows")]
    return "exe";
    
    #[cfg(not(target_os = "windows"))]
    return "";
}

/// Returns whether the given path points to a binary file
pub fn is_binary(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        #[cfg(target_os = "windows")]
        return extension == "exe";
        
        #[cfg(not(target_os = "windows"))]
        {
            // On Unix systems, check if the file is executable
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = path.metadata() {
                return metadata.permissions().mode() & 0o111 != 0;
            }
        }
    }
    false
}

/// Make a file executable
pub fn make_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

/// Expand environment variables in a path string
pub fn expand_path(path: &str) -> String {
    let mut result = path.to_string();
    
    if let Ok(home) = std::env::var("HOME") {
        result = result.replace("$HOME", &home);
        result = result.replace("~", &home);
    }
    
    #[cfg(windows)]
    {
        use std::env;
        for (key, value) in env::vars() {
            result = result.replace(&format!("%{}%", key), &value);
        }
    }
    
    result
}

/// Create a symlink
#[cfg(unix)]
pub fn create_symlink(src: &Path, dst: &Path) -> Result<()> {
    std::os::unix::fs::symlink(src, dst)?;
    Ok(())
}

#[cfg(windows)]
pub fn create_symlink(src: &Path, dst: &Path) -> Result<()> {
    if is_binary(src) {
        std::os::windows::fs::symlink_file(src, dst)?;
    } else {
        std::os::windows::fs::symlink_dir(src, dst)?;
    }
    Ok(())
}

