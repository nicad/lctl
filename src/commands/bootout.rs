use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::Result;
use crate::launchctl;
use crate::output::stdout;

pub fn run(angel: &Angel, args: &NameArgs) -> Result<()> {
    let daemon = angel.daemons.get_match(&args.name, args.exact)?;
    let result = launchctl::bootout(daemon)?;
    stdout::write(&result.stdout);
    result.require_ok(&format!("boot out {}", daemon.name))?;
    stdout::success(&format!("booted out {}", daemon.name));
    Ok(())
}
