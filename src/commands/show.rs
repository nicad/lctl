use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::{AngelError, Result};
use crate::output::stdout;

pub fn run(angel: &Angel, args: &NameArgs) -> Result<()> {
    let daemon = angel.daemons.get_match(&args.name, args.exact)?;
    let path = match daemon.source_path.as_ref() {
        Some(path) => path,
        None => return Err(AngelError::Launchctl("Daemon has no source path".to_string())),
    };
    let content = std::fs::read_to_string(path)?;
    stdout::write(&content);
    Ok(())
}
