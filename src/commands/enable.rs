use crate::cli::MultiNameArgs;
use crate::error::Result;
use crate::launchctl;
use crate::lctl::Lctl;
use crate::types::Daemon;

pub(crate) fn run(lctl: &Lctl, args: &MultiNameArgs) -> Result<()> {
    super::run_steps(
        lctl,
        args,
        |daemon| {
            // bootstrap of a disabled service fails, so enable first
            super::step(daemon, launchctl::enable, "enable")?;
            super::ensure_loaded(&Daemon { disabled: false, ..daemon.clone() })
        },
        "enabled",
    )
}
