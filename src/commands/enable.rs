use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::{AngelError, Result};
use crate::launchctl;
use crate::output::stdout;

pub fn run(angel: &Angel, args: &NameArgs) -> Result<()> {
    let daemon = angel.daemons.get_match(&args.name, args.exact)?;
    let result = launchctl::enable(daemon)?;
    stdout::write(&result.output);
    if result.success() {
        stdout::success(&format!("enabled {}", daemon.name));
        Ok(())
    } else {
        Err(AngelError::Launchctl(format!(
            "failed to enable {}: {}",
            daemon.name, result.stderr
        )))
    }
}
