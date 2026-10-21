use crate::cli::NameArgs;
use crate::error::Result;
use crate::lctl::Lctl;
use crate::output::stdout;

pub(crate) fn run(lctl: &Lctl, args: &NameArgs) -> Result<()> {
    let daemon = super::resolve_daemon(lctl, &args.name, args.exact)?;
    let path = daemon.require_source_path()?;
    let content = std::fs::read_to_string(path)?;
    stdout::write(&content);
    Ok(())
}
