use crate::error::Result;
use colored::Colorize;
use std::path::Path;

#[cfg(windows)]
pub async fn add_to_path(path: &Path) -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    println!("{} Adding packages directory to PATH...", "→".blue());
    
    // Open the environment key
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| anyhow::anyhow!("Failed to open Environment registry key: {}", e))?;
    
    // Get current PATH
    let current_path: String = environment.get_value("Path")
        .map_err(|e| anyhow::anyhow!("Failed to get current PATH: {}", e))?;
    
    // Check if our directory is already in PATH
    let new_dir = path.to_string_lossy().into_owned();
    if !current_path.split(';').any(|p| p == new_dir) {
        // Add our directory to PATH
        let new_path = if current_path.ends_with(';') {
            format!("{}{}", current_path, new_dir)
        } else {
            format!("{};{}", current_path, new_dir)
        };
        
        environment.set_value("Path", &new_path)
            .map_err(|e| anyhow::anyhow!("Failed to update PATH: {}", e))?;

        // Notify Windows of the environment change
        unsafe {
            use winapi::um::winuser::{HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG, SendMessageTimeoutW};
            use winapi::shared::minwindef::LPARAM;
            
            let mut wide_env: Vec<u16> = "Environment\0".encode_utf16().collect();
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                wide_env.as_ptr() as LPARAM,
                SMTO_ABORTIFHUNG,
                5000,
                std::ptr::null_mut(),
            );
        }
        
        println!("{} Added to PATH: {}", "✓".green(), new_dir);
        println!("{} You may need to restart your terminal for changes to take effect", "!".yellow());
    } else {
        println!("{} Directory already in PATH", "✓".green());
    }
    
    Ok(())
}

#[cfg(unix)]
pub async fn add_to_path(path: &Path) -> Result<()> {
    use std::env;
    use std::io::Write;
    
    let home = env::var("HOME")
        .map_err(|_| anyhow::anyhow!("Failed to get HOME directory"))?;
    
    let shell = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
    
    let shell_rc = if shell.contains("bash") {
        format!("{}/.bashrc", home)
    } else if shell.contains("zsh") {
        format!("{}/.zshrc", home)
    } else {
        format!("{}/.profile", home)
    };

    let export_line = format!("\nexport PATH=\"{}:$PATH\"", path.to_string_lossy());
    
    let rc_content = std::fs::read_to_string(&shell_rc)
        .unwrap_or_else(|_| String::new());
    
    if !rc_content.contains(&export_line) {
        std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&shell_rc)?
            .write_all(export_line.as_bytes())?;

        println!("{} Added to PATH in {}", "✓".green(), shell_rc);
        println!("{} Run 'source {}' or restart your terminal for changes to take effect", "!".yellow(), shell_rc);
    } else {
        println!("{} Directory already in PATH", "✓".green());
    }
    
    Ok(())
}