pub mod angel;
pub mod cli;
pub mod commands;
pub mod config;
pub mod daemon;
pub mod error;
pub mod launchctl;
pub mod output;
pub mod parser;
pub mod styles;
pub mod types;
pub mod utils;

use clap::Parser;
use cli::{Cli, Commands};

pub fn run() -> Result<(), error::Error> {
    let cli = Cli::parse();

    output::init(cli.verbose);

    if let Commands::Cheatsheet { subcommand } = &cli.command {
        commands::cheatsheet::run(subcommand);
        return Ok(());
    }

    let angel = angel::Angel::load()?;

    match cli.command {
        Commands::Start(args) => commands::start::run(&angel, &args),
        Commands::Stop(args) => commands::stop::run(&angel, &args),
        Commands::Restart(args) => commands::restart::run(&angel, &args),
        Commands::Reboot(args) => commands::reboot::run(&angel, &args),
        Commands::Status(args) => commands::status::run(&angel, &args),
        Commands::Logs(args) => commands::logs::run(&angel, &args),
        Commands::List(args) => commands::list::run(&angel, &args),
        Commands::Plist(args) => commands::show::run(&angel, &args),
        Commands::Install(args) => commands::install::run(&angel, &args),
        Commands::Uninstall(args) => commands::uninstall::run(&angel, &args),
        Commands::Bootstrap(args) => commands::bootstrap::run(&angel, &args),
        Commands::Bootout(args) => commands::bootout::run(&angel, &args),
        Commands::Enable(args) => commands::enable::run(&angel, &args),
        Commands::Disable(args) => commands::disable::run(&angel, &args),
        Commands::Print(args) => commands::print::run(&angel, &args),
        Commands::Cheatsheet { .. } => unreachable!(),
    }
}
