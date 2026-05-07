use std::os::unix::fs::symlink;

use anyhow::{bail, Result};

use super::LinkArgs;
use crate::config::Config;
use crate::utils::{detect_project, get_tasky_dir};

pub fn run(args: LinkArgs) -> Result<()> {
    // 1. Load config
    let config = Config::ensure_loaded()?;

    // 2. Detect project
    let project = match args.project {
        Some(p) => p,
        None => detect_project()?,
    };

    // 3. Get the vault project dir
    let vault_dir = get_tasky_dir(&config.vault.path, &project);

    // 4. Ensure vault project dir exists
    if !vault_dir.exists() {
        std::fs::create_dir_all(&vault_dir)?;
    }

    // 5. Define symlink path: <current_dir>/_tasky
    let current_dir = std::env::current_dir()?;
    let link_path = current_dir.join("_tasky");

    // 6. Check if symlink already exists
    if link_path.exists() || link_path.symlink_metadata().is_ok() {
        if let Ok(existing_target) = std::fs::read_link(&link_path) {
            if existing_target == vault_dir {
                println!("Link already exists: _tasky → {}", vault_dir.display());
                return Ok(());
            } else {
                bail!(
                    "_tasky already exists but points to {} (expected {}). Remove it manually if you want to relink.",
                    existing_target.display(),
                    vault_dir.display()
                );
            }
        } else {
            bail!(
                "_tasky already exists and is not a symlink. Remove it manually if you want to relink."
            );
        }
    }

    // 7. Create the symlink
    symlink(&vault_dir, &link_path)?;

    // 8. Handle .gitignore
    let gitignore_path = current_dir.join(".gitignore");
    if gitignore_path.exists() {
        let content = std::fs::read_to_string(&gitignore_path)?;
        let already_ignored = content.lines().any(|line| line.trim() == "_tasky");
        if !already_ignored {
            let mut new_content = content;
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push_str("_tasky\n");
            std::fs::write(&gitignore_path, new_content)?;
            println!("Added _tasky to .gitignore");
        }
    }

    // 9. Print confirmation
    println!("Linked: _tasky → {}", vault_dir.display());

    Ok(())
}
