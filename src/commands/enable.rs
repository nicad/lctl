use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::Result;
use crate::launchctl;

pub fn run(angel: &Angel, args: &NameArgs) -> Result<()> {
    super::run_launchctl(angel, args, launchctl::enable, "enable", "enabled")
}
