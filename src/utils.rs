use crate::error::{Error, Result, err_msg};
use std::path::PathBuf;
use std::sync::OnceLock;

static HOME: OnceLock<PathBuf> = OnceLock::new();

/// Initialize HOME from env. Call once at startup (e.g. in Angel::load()).
pub fn init_home() -> Result<()> {
    let path = std::env::var("HOME")
        .map_err(|_| Error::Config(err_msg!("HOME environment variable not set")))
        .map(PathBuf::from)?;
    HOME.set(path).map_err(|_| Error::Config(err_msg!("HOME already initialized")))?;
    Ok(())
}

/// Reference to the home directory. Panics if init_home() was not called first.
pub fn home() -> &'static PathBuf {
    HOME.get().expect("HOME not initialized; call init_home() (e.g. via Angel::load()) first")
}

/// Expands a leading `~` or `~/…` using [`home`]; other paths pass through unchanged.
pub fn expand_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        home().join(rest)
    } else if path == "~" {
        home().clone()
    } else {
        PathBuf::from(path)
    }
}
