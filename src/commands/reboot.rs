use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::{Error, Result, err_msg};
use crate::launchctl;
use crate::output::stdout;

pub fn run(angel: &Angel, args: &NameArgs) -> Result<()> {
    let daemon = angel.daemons.get_match(&args.name, args.exact)?;
    if daemon.source_path.is_none() {
        return Err(Error::InvalidInput(err_msg!(
            "`{}` has no plist path on disk (cannot reboot without a known plist)",
            daemon.name
        )));
    }

    let out = launchctl::bootout(daemon)?;
    stdout::write(&out.stdout);
    out.require_ok(&format!("boot out {}", daemon.name))?;

    let loaded = launchctl::bootstrap(daemon)?;
    stdout::write(&loaded.stdout);
    loaded.require_ok(&format!("bootstrap {}", daemon.name))?;

    stdout::success(&format!("rebooted {}", daemon.name));
    Ok(())
}
