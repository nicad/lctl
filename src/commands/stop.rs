use crate::cli::MultiNameArgs;
use crate::error::Result;
use crate::lctl::Lctl;

pub(crate) fn run(lctl: &Lctl, args: &MultiNameArgs) -> Result<()> {
    super::run_steps(lctl, args, super::ensure_unloaded, "stopped")
}
