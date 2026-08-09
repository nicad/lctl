use crate::error::{Error, Result, err_msg};
use crate::types::Domain;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub directories: Option<Vec<DirectoryConfig>>,
    pub colors: Option<Colors>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DirectoryConfig {
    pub path: String,
    #[serde(with = "domain_serde")]
    pub domain: Domain,
}

mod domain_serde {
    use super::Domain;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(domain: &Domain, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match domain {
            Domain::System => "system",
            Domain::User(_) => "user",
            Domain::Gui(_) => "gui",
            Domain::Unknown => "unknown",
        };
        serializer.serialize_str(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Domain, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Return Domain without uid - uid will be added when creating PlistDir
        match s.as_str() {
            "system" => Ok(Domain::System),
            "user" => Ok(Domain::User(0)), // Placeholder, will be replaced
            "gui" => Ok(Domain::Gui(0)),   // Placeholder, will be replaced
            "unknown" => Ok(Domain::Unknown),
            _ => Ok(Domain::Unknown),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct Colors {
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub title: Option<String>,
    pub section_header: Option<String>,
    pub command: Option<String>,
    pub argument: Option<String>,
    pub flag: Option<String>,
    pub table_border: Option<String>,
}

impl Config {
    pub fn load() -> Result<Config> {
        // Skip config loading if running as root
        let home = crate::utils::home();
        if home.as_path() == std::path::Path::new("/var/root") {
            return Ok(Config { directories: None, colors: None });
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
                    let home_str = home.to_string_lossy().to_string();
                    for dir in dirs.iter_mut() {
                        if dir.path.starts_with('~') {
                            dir.path = dir.path.replacen('~', &home_str, 1);
                        }
                    }
                }

                return Ok(config);
            }
        }

        // Config file is optional
        Ok(Config { directories: None, colors: None })
    }

    pub fn get_directories(&self) -> Vec<DirectoryConfig> {
        self.directories.clone().unwrap_or_default()
    }
}
