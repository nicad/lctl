use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::Result;
use crate::launchctl;

pub(crate) fn run(angel: &Angel, args: &NameArgs) -> Result<()> {
    super::run_launchctl(angel, args, launchctl::kickstart_kill, "restart", "restarted")
}
