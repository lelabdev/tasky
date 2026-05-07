use anyhow::Result;
use std::io::{self, Write};

use crate::config::Config;

pub fn run() -> Result<()> {
    // If config already exists, display it and exit
    if Config::exists()? {
        let config = Config::load()?;
        println!("Configuration already exists at {:?}\n", Config::config_path()?);
        println!("{}", toml::to_string_pretty(&config)?);
        return Ok(());
    }

    println!("tasky init — setting up configuration...\n");

    // Prompt for vault path with default
    let default_vault = dirs::home_dir()
        .map(|p| p.join("obsidian").display().to_string())
        .unwrap_or_else(|| "~/obsidian".to_string());

    print!("Vault path [{}]: ", default_vault);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let vault_path = input.trim();

    let resolved_path = if vault_path.is_empty() {
        default_vault.clone()
    } else {
        // Expand ~ to home directory if present
        if vault_path.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                vault_path.replacen(
                    '~',
                    &home.display().to_string(),
                    1,
                )
            } else {
                vault_path.to_string()
            }
        } else {
            vault_path.to_string()
        }
    };

    // Build config with defaults and user-provided vault path
    let mut config = Config::default();
    config.vault.path = resolved_path;

    // Save the config
    config.save()?;

    println!("\nConfiguration saved to {:?}", Config::config_path()?);
    println!("\n{}", toml::to_string_pretty(&config)?);
    println!("Tasky is ready to use!");

    Ok(())
}
