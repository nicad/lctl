use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::Result;
use crate::launchctl;
use crate::output::stdout;

pub(crate) fn run(angel: &Angel, args: &NameArgs) -> Result<()> {
    let daemon = super::resolve_daemon(angel, &args.name, args.exact)?;
    daemon.require_source_path()?;

    let out = launchctl::bootout(daemon)?;
    stdout::write(&out.stdout);
    out.require_ok(&format!("boot out {}", daemon.name))?;

    let loaded = launchctl::bootstrap(daemon)?;
    stdout::write(&loaded.stdout);
    loaded.require_ok(&format!("bootstrap {}", daemon.name))?;

    stdout::success(&format!("rebooted {}", daemon.name));
    Ok(())
}
