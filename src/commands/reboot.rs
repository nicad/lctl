use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::{AngelError, Result};
use crate::launchctl;
use crate::output::stdout;

pub fn run(angel: &Angel, args: &NameArgs) -> Result<()> {
    let daemon = angel.daemons.get_match(&args.name, args.exact)?;
    if daemon.source_path.is_none() {
        return Err(AngelError::InvalidArgument(format!(
            "`{}` has no plist path on disk (cannot reboot without a known plist)",
            daemon.name
        ))
        .into());
    }

    let out = launchctl::bootout(daemon)?;
    stdout::write(&out.output);
    if !out.success() {
        return Err(AngelError::Launchctl(format!(
            "failed to boot out {}: {}",
            daemon.name, out.stderr
        )));
    }

    let loaded = launchctl::bootstrap(daemon)?;
    stdout::write(&loaded.output);
    if !loaded.success() {
        return Err(AngelError::Launchctl(format!(
            "booted out {} but bootstrap failed: {}",
            daemon.name, loaded.stderr
        )));
    }

    stdout::success(&format!("rebooted {}", daemon.name));
    Ok(())
}
