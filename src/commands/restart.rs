use crate::cli::MultiNameArgs;
use crate::error::Result;
use crate::launchctl;
use crate::lctl::Lctl;

pub(crate) fn run(lctl: &Lctl, args: &MultiNameArgs) -> Result<()> {
    super::run_steps(
        lctl,
        args,
        |daemon| {
            if daemon.loaded {
                return super::step(daemon, launchctl::kickstart_kill, "restart");
            }
            // -k right after bootstrap would kill a process RunAtLoad just started
            super::ensure_loaded(daemon)?;
            super::step(daemon, launchctl::kickstart, "restart")
        },
        "restarted",
    )
}
