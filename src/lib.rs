pub(crate) mod angel;
pub(crate) mod cli;
pub(crate) mod commands;
pub(crate) mod config;
pub(crate) mod daemon;
pub(crate) mod error;
pub(crate) mod launchctl;
pub(crate) mod output;
pub(crate) mod parser;
pub(crate) mod styles;
pub(crate) mod types;
pub(crate) mod utils;

use angel::Angel;
use clap::Parser;
use cli::{Cli, Commands};
use error::Result;

pub use error::Error;

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    output::init(cli.verbose);

    match cli.command {
        Commands::Cheatsheet { subcommand } => commands::cheatsheet::run(&subcommand),
        command => dispatch(&Angel::load()?, command),
    }
}

fn dispatch(angel: &Angel, command: Commands) -> Result<()> {
    match command {
        Commands::Start(args) => commands::start::run(angel, &args),
        Commands::Stop(args) => commands::stop::run(angel, &args),
        Commands::Restart(args) => commands::restart::run(angel, &args),
        Commands::Reboot(args) => commands::reboot::run(angel, &args),
        Commands::Status(args) => commands::status::run(angel, &args),
        Commands::Logs(args) => commands::logs::run(angel, &args),
        Commands::List(args) => commands::list::run(angel, &args),
        Commands::Plist(args) => commands::show::run(angel, &args),
        Commands::Install(args) => commands::install::run(angel, &args),
        Commands::Uninstall(args) => commands::uninstall::run(angel, &args),
        Commands::Bootstrap(args) => commands::bootstrap::run(angel, &args),
        Commands::Bootout(args) => commands::bootout::run(angel, &args),
        Commands::Enable(args) => commands::enable::run(angel, &args),
        Commands::Disable(args) => commands::disable::run(angel, &args),
        Commands::Print(args) => commands::print::run(angel, &args),
        Commands::Cheatsheet { .. } => {
            // Handled in run() before Angel::load.
            Ok(())
        }
    }
}

/// Print a user-facing error to stderr (styled).
pub fn print_error(e: &Error) {
    output::stderr::error(&e.to_string());
}
