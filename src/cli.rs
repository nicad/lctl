use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "lctl")]
#[command(about = "A simpler CLI on top of macOS launchctl", version = env!("CARGO_PKG_VERSION"))]
#[command(
    after_help = "Run `lctl explain` to see the launchctl commands behind each lctl command.",
    mut_subcommands = explain_hint
)]
pub struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
    /// Assume yes for confirmation prompts
    #[arg(short = 'y', long = "yes", global = true)]
    pub yes: bool,
    /// No command: overview of your services (same as `lctl list`)
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Ends each subcommand's help with where to see what it runs.
fn explain_hint(subcommand: clap::Command) -> clap::Command {
    let hint = format!("Under the hood: lctl explain {}", subcommand.get_name());
    if subcommand.get_name() == "explain" { subcommand } else { subcommand.after_help(hint) }
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
pub struct OptionalNameArgs {
    /// Service name (omit for all of your services)
    pub name: Option<String>,
    /// Exact match
    #[arg(short, long)]
    pub exact: bool,
}

#[derive(Clone, Copy, Default, ValueEnum)]
pub enum SortBy {
    Parent,
    Domain,
    #[default]
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

#[derive(Args, Default)]
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
    /// Only running services (state filters can be combined)
    #[arg(long)]
    pub running: bool,
    /// Only loaded services that are not running
    #[arg(long)]
    pub idle: bool,
    /// Only services whose last run exited with an error
    #[arg(long)]
    pub failed: bool,
    /// Only services that are not loaded
    #[arg(long)]
    pub not_loaded: bool,
    /// Only disabled services
    #[arg(long)]
    pub disabled: bool,
    /// Also show the Domain and Plist columns
    #[arg(short, long)]
    pub long: bool,
    /// Print labels only, one per line
    #[arg(short = '1', long = "labels")]
    pub labels_only: bool,
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
    pub target: OptionalNameArgs,
    /// Also show all launchd fields and the raw launchctl print output
    #[arg(short = 'd', long)]
    pub debug: bool,
    /// Without a name: also show the Domain and Plist columns
    #[arg(short, long)]
    pub long: bool,
}

#[derive(Args)]
pub struct KillArgs {
    #[command(flatten)]
    pub target: NameArgs,
    /// Signal name (TERM, HUP, KILL…, with or without SIG) or number
    #[arg(short, long, default_value = "TERM")]
    pub signal: String,
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
    /// Send a signal to a service's process (default TERM); it stays loaded
    Kill(KillArgs),
    /// Show a service's state, config and next steps (no name: overview)
    Status(StatusArgs),
    /// Show service logs
    Logs(LogsArgs),
    /// List services with their state and when they run
    #[command(alias = "ls")]
    List(ListArgs),
    /// Print a service's plist
    Plist(NameArgs),
    /// Enable a service and load it (persists across reboots)
    Enable(NameArgs),
    /// Disable a service and unload it (persists across reboots)
    Disable(NameArgs),
    /// Check plists and services for common problems (no name: all of yours)
    Doctor(OptionalNameArgs),
    /// Show the launchctl commands behind lctl commands, and the reverse
    Explain {
        /// An lctl or launchctl command, e.g. `stop` or `bootout` (omit for everything)
        command: Option<String>,
    },
    /// Generate shell completions
    Completion {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
}
