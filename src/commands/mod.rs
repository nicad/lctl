pub(crate) mod disable;
pub(crate) mod enable;
pub(crate) mod install;
pub(crate) mod list;
pub(crate) mod logs;
pub(crate) mod reload;
pub(crate) mod restart;
pub(crate) mod show;
pub(crate) mod start;
pub(crate) mod status;
pub(crate) mod stop;
pub(crate) mod uninstall;

use crate::cli::NameArgs;
use crate::error::Result;
use crate::launchctl::{self, LaunchctlOutput};
use crate::lctl::Lctl;
use crate::output::stdout;
use crate::types::Daemon;
use crate::utils;

/// Resolve a daemon by name (prompts when multiple fuzzy matches).
pub(crate) fn resolve_daemon<'a>(lctl: &'a Lctl, query: &str, exact: bool) -> Result<&'a Daemon> {
    utils::pick_daemon(lctl.daemons.get_matches(query, exact), query)
}

/// Resolve among lctl-managed daemons only (prompts when multiple matches).
pub(crate) fn resolve_lctl_daemon<'a>(lctl: &'a Lctl, query: &str) -> Result<&'a Daemon> {
    utils::pick_daemon(lctl.daemons.get_lctl_matches(query), query)
}

/// Resolve a named daemon, run `steps` on it, then report `done`.
pub(crate) fn run_steps(
    lctl: &Lctl,
    args: &NameArgs,
    steps: impl FnOnce(&Daemon) -> Result<()>,
    done: &str,
) -> Result<()> {
    let daemon = resolve_daemon(lctl, &args.name, args.exact)?;
    steps(daemon)?;
    stdout::success(&format!("{done} {}", daemon.name));
    Ok(())
}

/// Run one launchctl op, print its stdout, require success.
pub(crate) fn step(
    daemon: &Daemon,
    op: impl FnOnce(&Daemon) -> Result<LaunchctlOutput>,
    action: &str,
) -> Result<()> {
    let result = op(daemon)?;
    stdout::write(&result.stdout);
    result.require_ok(&format!("{action} {}", daemon.name))?;
    Ok(())
}

/// Bootstrap unless launchd already has the service loaded.
pub(crate) fn ensure_loaded(daemon: &Daemon) -> Result<()> {
    if daemon.loaded { Ok(()) } else { step(daemon, launchctl::bootstrap, "bootstrap") }
}

/// Boot out if launchd has the service loaded.
pub(crate) fn ensure_unloaded(daemon: &Daemon) -> Result<()> {
    if daemon.loaded { step(daemon, launchctl::bootout, "boot out") } else { Ok(()) }
}

/// Disable, then boot out if loaded. Disabling first keeps KeepAlive from restarting it.
pub(crate) fn disable_and_unload(daemon: &Daemon) -> Result<()> {
    step(daemon, launchctl::disable, "disable")?;
    ensure_unloaded(daemon)
}
