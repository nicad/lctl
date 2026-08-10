use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt;

#[derive(Parser)]
#[command(name = "angel")]
#[command(about = "macOS launchd service manager", version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
    /// Assume yes for confirmation prompts
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,
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

#[derive(Clone, Copy, ValueEnum)]
pub enum SortBy {
    Parent,
    Domain,
    Name,
}

#[derive(Clone, ValueEnum, PartialEq)]
pub enum InstallStrategy {
    Copy,
    Symlink,
    Move,
}

#[derive(Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum InstallDomain {
    Gui,
    User,
    System,
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
    #[arg(short = 'i', long = "idle")]
    pub show_idle: bool,
    /// Field to sort by
    #[arg(short = 's', long = "sort", default_value = "name")]
    pub sort_by: SortBy,
}

#[derive(Args)]
pub struct InstallArgs {
    /// Path to the service file, or a daemon name / bare `.plist` filename
    pub path: String,
    /// Make a hard copy of the file instead of a symlink
    #[arg(short, long, default_value = "symlink")]
    pub strategy: InstallStrategy,
    /// Target domain (skips interactive domain selection)
    #[arg(long)]
    pub domain: Option<InstallDomain>,
}

#[derive(Clone, ValueEnum)]
pub enum Signal {
    /// Graceful termination (default)
    #[value(name = "sigterm")]
    SigTerm,
    /// Force immediate termination
    #[value(name = "sigkill")]
    SigKill,
    /// Hangup signal (often used for reload)
    #[value(name = "sighup")]
    SigHup,
    /// Interrupt signal
    #[value(name = "sigint")]
    SigInt,
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Signal::SigTerm => write!(f, "sigterm"),
            Signal::SigKill => write!(f, "sigkill"),
            Signal::SigHup => write!(f, "sighup"),
            Signal::SigInt => write!(f, "sigint"),
        }
    }
}

#[derive(Args)]
pub struct StartArgs {
    #[command(flatten)]
    pub target: NameArgs,
    /// Kill existing instance before starting
    #[arg(short, long)]
    pub kill: bool,
}

#[derive(Args)]
pub struct StopArgs {
    #[command(flatten)]
    pub target: NameArgs,
    /// Signal to send
    #[arg(short, long, default_value = "sigterm")]
    pub signal: Signal,
}

#[derive(Args)]
pub struct StatusArgs {
    #[command(flatten)]
    pub target: NameArgs,
    /// Debug output (show raw launchctl output and parsed data)
    #[arg(short = 'd', long)]
    pub debug: bool,
}

#[derive(Args)]
pub struct LogsArgs {
    #[command(flatten)]
    pub target: NameArgs,
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
    /// Boot out then bootstrap (reload plist from disk; picks up symlink target changes)
    Reboot(NameArgs),
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
}
