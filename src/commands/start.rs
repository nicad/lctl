use crate::cli::NameArgs;
use crate::error::Result;
use crate::launchctl;
use crate::lctl::Lctl;

pub(crate) fn run(lctl: &Lctl, args: &NameArgs) -> Result<()> {
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
