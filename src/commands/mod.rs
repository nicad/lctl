pub(crate) mod bootout;
pub(crate) mod bootstrap;
pub(crate) mod disable;
pub(crate) mod enable;
pub(crate) mod install;
pub(crate) mod list;
pub(crate) mod logs;
pub(crate) mod print;
pub(crate) mod reboot;
pub(crate) mod restart;
pub(crate) mod show;
pub(crate) mod start;
pub(crate) mod status;
pub(crate) mod stop;
pub(crate) mod uninstall;

use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::Result;
use crate::launchctl::LaunchctlOutput;
use crate::output::stdout;
use crate::types::Daemon;
use crate::utils;

/// Resolve a daemon by name (prompts when multiple fuzzy matches).
pub(crate) fn resolve_daemon<'a>(angel: &'a Angel, query: &str, exact: bool) -> Result<&'a Daemon> {
    utils::pick_daemon(angel.daemons.get_matches(query, exact), query)
}

/// Resolve among angel-managed daemons only (prompts when multiple matches).
pub(crate) fn resolve_angel_daemon<'a>(angel: &'a Angel, query: &str) -> Result<&'a Daemon> {
    utils::pick_daemon(angel.daemons.get_angel_matches(query), query)
}

/// Resolve a named daemon, run a launchctl op, print stdout, require success.
pub(crate) fn run_launchctl(
    angel: &Angel,
    args: &NameArgs,
    op: impl FnOnce(&Daemon) -> Result<LaunchctlOutput>,
    action: &str,
    done: &str,
) -> Result<()> {
    let daemon = resolve_daemon(angel, &args.name, args.exact)?;
    let result = op(daemon)?;
    stdout::write(&result.stdout);
    result.require_ok(&format!("{action} {}", daemon.name))?;
    stdout::success(&format!("{done} {}", daemon.name));
    Ok(())
}
