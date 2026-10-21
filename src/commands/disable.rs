use crate::cli::NameArgs;
use crate::error::Result;
use crate::lctl::Lctl;

pub(crate) fn run(lctl: &Lctl, args: &NameArgs) -> Result<()> {
    super::run_steps(lctl, args, super::disable_and_unload, "disabled")
}
