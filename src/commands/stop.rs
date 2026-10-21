use crate::cli::StopArgs;
use crate::error::Result;
use crate::launchctl;
use crate::lctl::Lctl;
use crate::output::stdout;

pub(crate) fn run(lctl: &Lctl, args: &StopArgs) -> Result<()> {
    let daemon = super::resolve_daemon(lctl, &args.target.name, args.target.exact)?;
    let result = launchctl::kill(daemon, &args.signal.to_string())?;
    stdout::write(&result.stdout);
    result.require_ok(&format!("stop {}", daemon.name))?;
    stdout::success(&format!("stopped {}", daemon.name));
    Ok(())
}
