use clap::{Args, Parser, Subcommand, ValueEnum};
use strum::Display;

const VERSION: &str = "0.1.0";

#[derive(Parser)]
#[command(name = "angel")]
#[command(about = "macOS launchd service manager", version = VERSION)]
pub struct Cli {
    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args)]
pub struct NameArgs {
    /// Service name
    pub name: String,
    /// Exact match
    #[arg(short, long)]
    pub exact: bool,
}

#[derive(Args)]
pub struct ListArgs {
    /// Pattern to match
    pub pattern: Option<String>,
    /// Exact match
    #[arg(short, long)]
    pub exact: bool,
    /// Show Apple daemons
    #[arg(short = 'a', long = "apple")]
    pub show_apple: bool,
    /// Show dynamically loaded daemons
    #[arg(short = 'd', long = "dynamic")]
    pub show_dynamic: bool,
    /// Show idle daemons (have no pid)
    #[arg(short = 'i', long = "idle", default_value = "false")]
    pub show_idle: bool,
    /// Field to sort by
    #[arg(short = 's', long = "sort", default_value = "name")]
    pub sort_by: crate::commands::list::SortBy,
}

#[derive(Args)]
pub struct InstallArgs {
    /// Path to the service file
    pub path: String,
    /// Make a hard copy of the file instead of a symlink
    #[arg(short, long, default_value = "symlink")]
    pub strategy: crate::commands::install::InstallStrategy,
}

#[derive(Clone, ValueEnum, Display)]
#[allow(non_camel_case_types)]
pub enum Signal {
    /// Graceful termination (default)
    sigterm,
    /// Force immediate termination
    sigkill,
    /// Hangup signal (often used for reload)
    sighup,
    /// Interrupt signal
    sigint,
}

#[derive(Args)]
pub struct StartArgs {
    /// Service name
    pub name: String,
    /// Exact match
    #[arg(short, long)]
    pub exact: bool,
    /// Kill existing instance before starting
    #[arg(short, long)]
    pub kill: bool,
}

#[derive(Args)]
pub struct StopArgs {
    /// Service name
    pub name: String,
    /// Exact match
    #[arg(short, long)]
    pub exact: bool,
    /// Signal to send
    #[arg(short, long, default_value = "sigterm")]
    pub signal: Signal,
    /// Force kill (equivalent to --signal sigkill)
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct StatusArgs {
    /// Service name
    pub name: String,
    /// Exact match
    #[arg(short, long)]
    pub exact: bool,
    /// Debug output (show raw launchctl output and parsed data)
    #[arg(short = 'd', long)]
    pub debug: bool,
}

#[derive(Args)]
pub struct LogsArgs {
    /// Service name
    pub name: String,
    /// Exact match
    #[arg(short, long)]
    pub exact: bool,
    /// Time frame for logs (e.g., 5m, 1h, 24h). Default: 5m
    #[arg(short = 't', long = "time", default_value = "5m")]
    pub time_frame: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a service
    Install(InstallArgs),
    /// Uninstall a service
    Uninstall(NameArgs),
    /// Start a service
    #[command(alias = "kickstart")]
    Start(StartArgs),
    /// Stop a service
    #[command(alias = "kill")]
    Stop(StopArgs),
    /// Restart a service
    #[command(alias = "kkill")]
    Restart(NameArgs),
    /// Bootstrap a service
    Bootstrap(NameArgs),
    /// Bootout a service
    Bootout(NameArgs),
    /// Show service status
    Status(StatusArgs),
    /// Show service logs
    Logs(LogsArgs),
    /// List services
    #[command(alias = "ls")]
    List(ListArgs),
    /// print a service's plist
    Plist(NameArgs),
    /// Enable a service
    Enable(NameArgs),
    /// Disable a service
    Disable(NameArgs),
    /// Print service information (launchctl print wrapper)
    Print(NameArgs),
    /// Show version
    Version,
    /// Show helpful reference information
    Cheatsheet {
        #[command(subcommand)]
        subcommand: crate::commands::cheatsheet::CheatsheetSubcommand,
    },
}
