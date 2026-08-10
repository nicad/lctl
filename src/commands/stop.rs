use crate::angel::Angel;
use crate::cli::StopArgs;
use crate::error::Result;
use crate::launchctl;
use crate::output::stdout;

pub(crate) fn run(angel: &Angel, args: &StopArgs) -> Result<()> {
    let daemon = super::resolve_daemon(angel, &args.target.name, args.target.exact)?;
    let result = launchctl::kill(daemon, &args.signal.to_string())?;
    stdout::write(&result.stdout);
    result.require_ok(&format!("stop {}", daemon.name))?;
    stdout::success(&format!("stopped {}", daemon.name));
    Ok(())
}
