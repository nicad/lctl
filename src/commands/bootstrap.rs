use crate::cli::NameArgs;
use crate::error::Result;
use crate::launchctl;
use crate::lctl::Lctl;

pub(crate) fn run(lctl: &Lctl, args: &NameArgs) -> Result<()> {
    super::run_launchctl(lctl, args, launchctl::bootstrap, "bootstrap", "bootstrapped")
}
