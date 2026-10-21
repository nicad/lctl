use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "lctl")]
#[command(about = "A simpler CLI on top of macOS launchctl", version = env!("CARGO_PKG_VERSION"))]
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
    /// Also show Apple services
    #[arg(short = 'a', long = "apple")]
    pub show_apple: bool,
    /// Also show services without a plist on disk
    #[arg(short = 'd', long = "dynamic")]
    pub show_dynamic: bool,
    /// Also show services that are not running
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
    /// Start a service (loads it first if needed)
    Start(NameArgs),
    /// Stop and unload a service (it stays stopped until next login/boot)
    Stop(NameArgs),
    /// Restart a service (loads it first if needed)
    Restart(NameArgs),
    /// Unload and load again, picking up plist edits and symlink target changes
    Reload(NameArgs),
    /// Show service status
    Status(StatusArgs),
    /// Show service logs
    Logs(LogsArgs),
    /// List services
    #[command(alias = "ls")]
    List(ListArgs),
    /// Print a service's plist
    Plist(NameArgs),
    /// Enable a service and load it (persists across reboots)
    Enable(NameArgs),
    /// Disable a service and unload it (persists across reboots)
    Disable(NameArgs),
    /// Generate shell completions
    Completion {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
}
