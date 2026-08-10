use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::Result;
use crate::launchctl;
use crate::output::stdout;

pub(crate) fn run(angel: &Angel, args: &NameArgs) -> Result<()> {
    let daemon = super::resolve_daemon(angel, &args.name, args.exact)?;
    let result = launchctl::print(&daemon.service_target())?;
    stdout::write(&result.stdout);
    Ok(())
}
