use crate::error::{Error, Result, err_msg};
use crate::styles;
use crate::types::Daemon;
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

/// Interactive yes/no prompt. Returns whether the user confirmed.
pub fn confirm(prompt: &str) -> Result<bool> {
    confirm_with_default(prompt, false)
}

/// Interactive yes/no prompt with an explicit default selection.
pub fn confirm_with_default(prompt: &str, default: bool) -> Result<bool> {
    Ok(dialoguer::Confirm::new().with_prompt(prompt).default(default).interact()?)
}

/// Pick one daemon from matches: empty → NotFound, one → that daemon, many → interactive select.
pub fn pick_daemon<'a>(matches: Vec<&'a Daemon>, query: &str) -> Result<&'a Daemon> {
    match matches.len() {
        0 => Err(Error::NotFound(err_msg!(query))),
        1 => Ok(matches[0]),
        _ => {
            let items: Vec<String> = matches
                .iter()
                .map(|daemon| {
                    format!(
                        "{:<19}{}  {}",
                        styles::color_domain(&daemon.domain),
                        daemon.name,
                        styles::command(&styles::display_path(daemon, false))
                    )
                })
                .collect();

            let selection = dialoguer::Select::new()
                .with_prompt(format!(
                    "Multiple daemons found matching '{}'. Select one:",
                    query
                ))
                .items(&items)
                .default(0)
                .interact()?;

            Ok(matches[selection])
        }
    }
}
