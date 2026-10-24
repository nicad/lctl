use crate::error::{Error, Result, err_msg};
use crate::styles;
use crate::types::Daemon;
use nix::unistd::{Uid, User};
use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::OnceLock;

static HOME: OnceLock<PathBuf> = OnceLock::new();

/// Home directory for the invoking user. Lazy-initialized.
///
/// Under `sudo`, uses the home of `SUDO_UID` from passwd (not `$HOME`, which is often
/// `/var/root`). Otherwise uses `$HOME`.
pub fn home() -> Result<&'static PathBuf> {
    if let Some(path) = HOME.get() {
        return Ok(path);
    }
    let path = resolve_home()?;
    let _ = HOME.set(path);
    HOME.get().ok_or_else(|| Error::Config(err_msg!("HOME failed to initialize")))
}

fn resolve_home() -> Result<PathBuf> {
    if let Ok(sudo_uid) = std::env::var("SUDO_UID") {
        if let Ok(uid) = sudo_uid.parse::<u32>() {
            if let Ok(Some(user)) = User::from_uid(Uid::from_raw(uid)) {
                return Ok(user.dir);
            }
        }
    }
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| Error::Config(err_msg!("HOME environment variable not set")))
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
    if crate::output::assume_yes() {
        return Ok(true);
    }
    require_terminal("add -y to answer yes")?;
    Ok(dialoguer::Confirm::new().with_prompt(prompt).default(default).interact()?)
}

/// dialoguer prompts on stderr; without a terminal there, say what to do instead of failing
/// with its "not a terminal".
fn require_terminal(instead: &str) -> Result<()> {
    if std::io::stderr().is_terminal() {
        Ok(())
    } else {
        Err(Error::InvalidInput(err_msg!("no terminal to ask on: {instead}")))
    }
}

/// Pick one daemon from matches: empty → NotFound, one → that daemon, many → interactive select.
/// Matches are sorted by name so the menu (and `-y` default) is stable.
pub fn pick_daemon<'a>(mut matches: Vec<&'a Daemon>, query: &str) -> Result<&'a Daemon> {
    matches.sort_by(|a, b| a.name.cmp(&b.name));
    match matches.len() {
        0 => Err(Error::NotFound(err_msg!(query))),
        1 => Ok(matches[0]),
        _ if crate::output::assume_yes() => Ok(matches[0]),
        _ => {
            require_terminal(&format!(
                "{} services match `{query}`; use -e with the full label, -A for all, or -y for the first",
                matches.len()
            ))?;
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
                .with_prompt(format!("Multiple daemons found matching '{}'. Select one:", query))
                .items(&items)
                .default(0)
                .interact()?;

            Ok(matches[selection])
        }
    }
}
