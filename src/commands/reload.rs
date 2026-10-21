use crate::cli::NameArgs;
use crate::error::Result;
use crate::launchctl;
use crate::lctl::Lctl;

pub(crate) fn run(lctl: &Lctl, args: &NameArgs) -> Result<()> {
    super::run_steps(
        lctl,
        args,
        |daemon| {
            // fail before unloading if there is no plist to load again
            daemon.require_source_path()?;
            super::ensure_unloaded(daemon)?;
            super::step(daemon, launchctl::bootstrap, "bootstrap")
        },
        "reloaded",
    )
}
