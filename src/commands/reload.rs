use crate::cli::MultiNameArgs;
use crate::error::Result;
use crate::launchctl;
use crate::lctl::Lctl;

pub(crate) fn run(lctl: &Lctl, args: &MultiNameArgs) -> Result<()> {
    super::run_steps(
        lctl,
        args,
        |daemon| {
            // fail before unloading if it can't be loaded again
            daemon.require_source_path()?;
            super::require_enabled(daemon)?;
            super::ensure_unloaded(daemon)?;
            super::step(daemon, launchctl::bootstrap, "bootstrap")
        },
        "reloaded",
    )
}
