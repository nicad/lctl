use crate::error::{Error, Result, err_msg};
use crate::types::Domain;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub directories: Option<Vec<DirectoryConfig>>,
}

/// Domain kind as written in `.angelrc` (no uid; uid filled at runtime).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigDomain {
    System,
    User,
    Gui,
    #[serde(other)]
    Unknown,
}

impl ConfigDomain {
    pub fn to_domain(self, uid: u32) -> Domain {
        match self {
            ConfigDomain::System => Domain::System,
            ConfigDomain::User => Domain::User(uid),
            ConfigDomain::Gui => Domain::Gui(uid),
            ConfigDomain::Unknown => Domain::Unknown,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DirectoryConfig {
    pub path: String,
    pub domain: ConfigDomain,
}

impl Config {
    pub fn load() -> Result<Config> {
        // Skip config loading if running as root
        let home = crate::utils::home();
        if home.as_path() == std::path::Path::new("/var/root") {
            return Ok(Config { directories: None });
        }

        // Try config file locations
        let config_paths = vec![
            home.join(".angelrc"),
            dirs::config_dir().map(|p| p.join("angel").join(".angelrc")).unwrap_or_default(),
            home.join(".config").join("angel").join(".angelrc"),
        ];

        for path in config_paths {
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                let mut config: Config = serde_yaml::from_str(&content).map_err(|e| {
                    Error::Config(err_msg!("Failed to parse config: {}", e))
                })?;

                // Expand ~ in directory paths
                if let Some(ref mut dirs) = config.directories {
                    for dir in dirs.iter_mut() {
                        dir.path = crate::utils::expand_path(&dir.path).display().to_string();
                    }
                }

                return Ok(config);
            }
        }

        // Config file is optional
        Ok(Config { directories: None })
    }

    pub fn get_directories(&self) -> Vec<DirectoryConfig> {
        self.directories.clone().unwrap_or_default()
    }
}
