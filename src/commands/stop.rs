use crate::angel::Angel;
use crate::cli::StopArgs;
use crate::error::Result;
use crate::launchctl;
use crate::output::stdout;

pub fn run(angel: &Angel, args: &StopArgs) -> Result<()> {
    let daemon = angel.daemons.get_match(&args.name, args.exact)?;
    let result = launchctl::kill(daemon, &args.signal.to_string())?;
    stdout::write(&result.output);
    match result.success() {
        true => stdout::success(&format!("stopped {}", daemon.name)),
        false => stdout::error(&format!("failed to stop {}: {}", daemon.name, result.stderr)),
    }
    Ok(())
}
