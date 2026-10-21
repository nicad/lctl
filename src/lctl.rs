use crate::config::Config;
use crate::daemon::DaemonRegistry;
use crate::error::Result;
use crate::types::{Domain, ForWhom};
use nix::unistd::{self, Uid};
use std::path::{Path, PathBuf};

pub struct Lctl {
    pub daemons: DaemonRegistry,
    pub lctl_dir: PathBuf,
    pub home: PathBuf,
    pub euid: Uid,
    pub uid: Uid,
}

impl Lctl {
    pub fn load() -> Result<Self> {
        let home = crate::utils::home()?.clone();
        let config = Config::load()?;
        let lctl_dir = get_lctl_dir()?;
        let euid = unistd::geteuid();
        // When running with sudo, getuid() may return 0. Check SUDO_UID first.
        let uid = std::env::var("SUDO_UID")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .map(Uid::from_raw)
            .unwrap_or_else(|| unistd::getuid());
        let daemons = DaemonRegistry::new(&config, uid.as_raw(), &lctl_dir, &home)?;

        Ok(Self { daemons, lctl_dir, home, euid, uid })
    }

    pub fn is_root(&self) -> bool {
        self.euid.is_root()
    }
}

pub fn get_lctl_dir() -> Result<PathBuf> {
    // Check LCTL_CONFIG_DIR first
    if let Ok(dir) = std::env::var("LCTL_CONFIG_DIR") {
        return Ok(PathBuf::from(dir));
    }

    // Fall back to the platform config dir (~/Library/Application Support on macOS)
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => crate::utils::home()?.join(".config"),
    };
    Ok(config_dir.join("lctl"))
}

/// lctl-managed plist roots: `{lctl_dir}/{user,system,gui}`.
pub fn lctl_domain_dirs(lctl_dir: &Path, uid: u32) -> [(PathBuf, Domain, ForWhom); 3] {
    [
        (lctl_dir.join("user"), Domain::User(uid), ForWhom::Lctl),
        (lctl_dir.join("system"), Domain::System, ForWhom::Lctl),
        (lctl_dir.join("gui"), Domain::Gui(uid), ForWhom::Lctl),
    ]
}
