use crate::error::{Error, Result, err_msg};
use crate::explain;
use crate::output::stdout;

pub(crate) fn run(command: Option<&str>) -> Result<()> {
    let text = match command {
        None => explain::everything(),
        Some(command) => explain::about(command).ok_or_else(|| {
            Error::InvalidInput(err_msg!(
                "nothing about `{command}`; `lctl explain` shows everything"
            ))
        })?,
    };
    stdout::write(&text);
    Ok(())
}
