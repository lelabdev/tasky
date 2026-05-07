use serde::{Deserialize, Serialize};

/// Tasky configuration stored at `~/.config/tasky/config.toml`
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub vault: VaultConfig,
    #[serde(default)]
    pub pomodoro: PomodoroConfig,
    #[serde(default)]
    pub sounds: SoundsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultConfig {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PomodoroConfig {
    #[serde(default = "default_work_duration")]
    pub work_duration: u64,
    #[serde(default = "default_short_break")]
    pub short_break: u64,
    #[serde(default = "default_long_break")]
    pub long_break: u64,
    #[serde(default = "default_long_break_interval")]
    pub long_break_interval: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SoundsConfig {
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub done: String,
    #[serde(default)]
    pub r#break: String,
}

fn default_work_duration() -> u64 {
    25
}
fn default_short_break() -> u64 {
    5
}
fn default_long_break() -> u64 {
    15
}
fn default_long_break_interval() -> u64 {
    4
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_duration: default_work_duration(),
            short_break: default_short_break(),
            long_break: default_long_break(),
            long_break_interval: default_long_break_interval(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir()
            .map(|p| p.join("obsidian").display().to_string())
            .unwrap_or_else(|| "~/obsidian".to_string());
        Self {
            vault: VaultConfig { path: home },
            pomodoro: PomodoroConfig::default(),
            sounds: SoundsConfig::default(),
        }
    }
}

impl Config {
    /// Load config from `~/.config/tasky/config.toml`.
    /// Returns default config if the file does not exist.
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path()?;
        if !config_path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load config, or prompt the user to run `tasky init` if missing.
    pub fn ensure_loaded() -> anyhow::Result<Self> {
        if Self::exists()? {
            Self::load()
        } else {
            anyhow::bail!(
                "Configuration not found. Run `tasky init` to set up Tasky."
            );
        }
    }

    /// Save config to `~/.config/tasky/config.toml`
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Check if config file exists
    pub fn exists() -> anyhow::Result<bool> {
        let path = Self::config_path()?;
        Ok(path.exists())
    }

    /// Get the config file path
    pub fn config_path() -> anyhow::Result<std::path::PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("cannot determine config directory"))?;
        Ok(config_dir.join("tasky").join("config.toml"))
    }
}
