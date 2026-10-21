use crate::cli::NameArgs;
use crate::error::Result;
use crate::launchctl;
use crate::lctl::Lctl;

pub(crate) fn run(lctl: &Lctl, args: &NameArgs) -> Result<()> {
    super::run_steps(
        lctl,
        args,
        |daemon| {
            // bootstrap of a disabled service fails, so enable first
            super::step(daemon, launchctl::enable, "enable")?;
            super::ensure_loaded(daemon)
        },
        "enabled",
    )
}
