use crate::config::Config;
use crate::daemon::DaemonRegistry;
use crate::error::Result;
use crate::types::{Domain, ForWhom};
use nix::unistd::{self, Uid};
use std::path::{Path, PathBuf};

pub struct Angel {
    pub daemons: DaemonRegistry,
    pub angel_dir: PathBuf,
    pub euid: Uid,
    pub uid: Uid,
}

impl Angel {
    pub fn load() -> Result<Self> {
        crate::utils::init_home()?;
        let config = Config::load()?;
        let angel_dir = get_angel_dir()?;
        let euid = unistd::geteuid();
        // When running with sudo, getuid() may return 0. Check SUDO_UID first.
        let uid = std::env::var("SUDO_UID")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .map(Uid::from_raw)
            .unwrap_or_else(|| unistd::getuid());
        let daemons = DaemonRegistry::new(&config, uid.as_raw(), &angel_dir)?;

        Ok(Self { daemons, angel_dir, euid, uid })
    }

    pub fn is_root(&self) -> bool {
        self.euid.is_root()
    }
}

pub fn get_angel_dir() -> Result<PathBuf> {
    // Check ANGEL_CONFIG_DIR first
    if let Ok(dir) = std::env::var("ANGEL_CONFIG_DIR") {
        return Ok(PathBuf::from(dir));
    }

    // Fall back to XDG_CONFIG_HOME or ~/.config
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => crate::utils::home().join(".config"),
    };
    Ok(config_dir.join("angel"))
}

/// Angel-managed plist roots: `{angel_dir}/{user,system,gui}`.
pub fn angel_domain_dirs(angel_dir: &Path, uid: u32) -> [(PathBuf, Domain, ForWhom); 3] {
    [
        (angel_dir.join("user"), Domain::User(uid), ForWhom::Angel),
        (angel_dir.join("system"), Domain::System, ForWhom::Angel),
        (angel_dir.join("gui"), Domain::Gui(uid), ForWhom::Angel),
    ]
}
