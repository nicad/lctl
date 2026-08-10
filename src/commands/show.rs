use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::Result;
use crate::output::stdout;

pub fn run(angel: &Angel, args: &NameArgs) -> Result<()> {
    let daemon = super::resolve_daemon(angel, &args.name, args.exact)?;
    let path = daemon.require_source_path()?;
    let content = std::fs::read_to_string(path)?;
    stdout::write(&content);
    Ok(())
}
