use crate::cli::NameArgs;
use crate::error::Result;
use crate::launchctl;
use crate::lctl::Lctl;
use crate::output::stdout;

pub(crate) fn run(lctl: &Lctl, args: &NameArgs) -> Result<()> {
    let daemon = super::resolve_daemon(lctl, &args.name, args.exact)?;
    let result = launchctl::print(&daemon.service_target())?;
    stdout::write(&result.stdout);
    Ok(())
}
