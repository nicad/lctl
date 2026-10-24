use crate::cli::MultiNameArgs;
use crate::error::Result;
use crate::launchctl;
use crate::lctl::Lctl;

pub(crate) fn run(lctl: &Lctl, args: &MultiNameArgs) -> Result<()> {
    super::run_steps(
        lctl,
        args,
        |daemon| {
            super::ensure_loaded(daemon)?;
            super::step(daemon, launchctl::kickstart, "start")
        },
        "started",
    )
}
