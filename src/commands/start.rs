use crate::angel::Angel;
use crate::cli::StartArgs;
use crate::error::{AngelError, Result};
use crate::launchctl;
use crate::output::stdout;

pub fn run(angel: &Angel, args: &StartArgs) -> Result<()> {
    let daemon = angel.daemons.get_match(&args.name, args.exact)?;
    let result = match args.kill {
        true => launchctl::kickstart_kill(daemon)?,
        false => launchctl::kickstart(daemon)?,
    };
    stdout::write(&result.output);
    if result.success() {
        stdout::success(&format!("started {}", daemon.name));
        Ok(())
    } else {
        Err(AngelError::Launchctl(format!(
            "failed to start {}: {}",
            daemon.name, result.stderr
        )))
    }
}
