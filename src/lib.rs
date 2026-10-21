pub mod cli;
pub(crate) mod commands;
pub mod config;
pub mod daemon;
pub mod describe;
pub mod doctor;
pub mod error;
pub mod launchctl;
pub mod lctl;
pub mod output;
pub mod parser;
pub mod styles;
pub mod sudo;
pub mod types;
pub mod utils;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands, ListArgs};
use error::Result;
use lctl::Lctl;

pub use error::Error;

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    output::init(cli.verbose, cli.yes);

    match cli.command {
        Some(Commands::Completion { shell }) => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "lctl", &mut std::io::stdout());
            Ok(())
        }
        Some(command) => dispatch(&Lctl::load()?, command),
        None => commands::list::run(&Lctl::load()?, &ListArgs::default()),
    }
}

fn dispatch(lctl: &Lctl, command: Commands) -> Result<()> {
    match command {
        Commands::Start(args) => commands::start::run(lctl, &args),
        Commands::Stop(args) => commands::stop::run(lctl, &args),
        Commands::Restart(args) => commands::restart::run(lctl, &args),
        Commands::Reload(args) => commands::reload::run(lctl, &args),
        Commands::Status(args) => commands::status::run(lctl, &args),
        Commands::Logs(args) => commands::logs::run(lctl, &args),
        Commands::List(args) => commands::list::run(lctl, &args),
        Commands::Plist(args) => commands::show::run(lctl, &args),
        Commands::Install(args) => commands::install::run(lctl, &args),
        Commands::Uninstall(args) => commands::uninstall::run(lctl, &args),
        Commands::Enable(args) => commands::enable::run(lctl, &args),
        Commands::Disable(args) => commands::disable::run(lctl, &args),
        Commands::Doctor(args) => commands::doctor::run(lctl, &args),
        Commands::Completion { .. } => unreachable!("handled in run()"),
    }
}

/// Print a user-facing error to stderr (styled).
pub fn print_error(e: &Error) {
    output::stderr::error(&e.to_string());
}
