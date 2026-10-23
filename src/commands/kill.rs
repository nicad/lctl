use crate::cli::KillArgs;
use crate::error::Result;
use crate::launchctl;
use crate::lctl::Lctl;

pub(crate) fn run(lctl: &Lctl, args: &KillArgs) -> Result<()> {
    let signal = &args.signal;
    super::run_steps(
        lctl,
        &args.target,
        |daemon| {
            super::require_running(daemon)?;
            super::step(
                daemon,
                |daemon| launchctl::kill(daemon, signal),
                &format!("send {signal} to"),
            )
        },
        &format!("sent {signal} to"),
    )
}
