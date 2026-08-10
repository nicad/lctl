use crate::error::{Error, Result, err_msg};
use crate::styles;
use crate::types::Daemon;
use std::path::PathBuf;
use std::sync::OnceLock;

static HOME: OnceLock<PathBuf> = OnceLock::new();

/// Home directory from `$HOME`. Lazy-initialized; returns Config error if unset.
pub fn home() -> Result<&'static PathBuf> {
    if let Some(path) = HOME.get() {
        return Ok(path);
    }
    let path = std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| Error::Config(err_msg!("HOME environment variable not set")))?;
    let _ = HOME.set(path);
    HOME.get().ok_or_else(|| Error::Config(err_msg!("HOME failed to initialize")))
}

/// Expands a leading `~` or `~/…` using [`home`]; other paths pass through unchanged.
pub fn expand_path(path: &str) -> Result<PathBuf> {
    if let Some(rest) = path.strip_prefix("~/") {
        Ok(home()?.join(rest))
    } else if path == "~" {
        Ok(home()?.clone())
    } else {
        Ok(PathBuf::from(path))
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
