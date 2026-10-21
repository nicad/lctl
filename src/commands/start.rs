use crate::cli::StartArgs;
use crate::error::Result;
use crate::launchctl;
use crate::lctl::Lctl;
use crate::output::stdout;

pub(crate) fn run(lctl: &Lctl, args: &StartArgs) -> Result<()> {
    let daemon = super::resolve_daemon(lctl, &args.target.name, args.target.exact)?;
    let result =
        if args.kill { launchctl::kickstart_kill(daemon)? } else { launchctl::kickstart(daemon)? };
    stdout::write(&result.stdout);
    result.require_ok(&format!("start {}", daemon.name))?;
    stdout::success(&format!("started {}", daemon.name));
    Ok(())
}
