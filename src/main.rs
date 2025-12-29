mod angel;
mod cli;
mod commands;
mod config;
mod daemon;
mod error;
mod launchctl;
mod output;
mod parser;
mod styles;
mod types;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use error::AngelError;

fn main() {
    let cli = Cli::parse();

    // Initialize output context before any commands run
    output::init(cli.verbose);

    // Load Angel instance before any command runs
    let angel = match angel::Angel::load() {
        Ok(angel) => angel,
        Err(e) => {
            match e {
                AngelError::User(_) => output::stdout::error(&e.to_string()),
                AngelError::System(_) => output::stderr::error(&e.to_string()),
            };
            std::process::exit(1);
        }
    };

    let result = match cli.command {
        Commands::Start(args) => commands::start::run(&angel, &args),
        Commands::Stop(args) => commands::stop::run(&angel, &args),
        Commands::Restart(args) => commands::restart::run(&angel, &args),
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
        Commands::Version => {
            commands::version::run();
            Ok(())
        }
        Commands::Cheatsheet { subcommand } => {
            commands::cheatsheet::run(&subcommand);
            Ok(())
        }
    };

    if let Err(e) = result {
        match e {
            AngelError::User(_) => output::stdout::error(&e.to_string()),
            AngelError::System(_) => output::stderr::error(&e.to_string()),
        }
        std::process::exit(1);
    }
}
