use crate::output::stdout;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum CheatsheetSubcommand {
    /// Show information about launchd domains
    Domains,
}

pub fn run(subcommand: &CheatsheetSubcommand) {
    match subcommand {
        CheatsheetSubcommand::Domains => print_domains(),
    }
}

fn print_domains() {
    let home = std::env::var("HOME").unwrap_or_default();

    stdout::writeln("Launchd Domains");
    stdout::writeln("");

    // System Domain
    stdout::writeln("System Domain");
    stdout::writeln("  Purpose: System-wide daemons that run as root, regardless of user login");
    stdout::writeln("  Use for: Services that need to run at system boot, before user login");
    stdout::writeln("  Files:");
    stdout::writeln("    • /System/Library/LaunchDaemons (Apple system daemons)");
    stdout::writeln("    • /Library/LaunchDaemons (third-party system daemons)");
    if !home.is_empty() {
        stdout::writeln(&format!(
            "    • {}/.config/angel/system (angel-managed system daemons)",
            home
        ));
    }
    stdout::writeln("");

    // User Domain
    stdout::writeln("User Domain");
    stdout::writeln("  Purpose: User-specific agents that run in the background, not tied to GUI");
    stdout::writeln(
        "  Use for: Services that should run when user logs in, but don't need GUI access",
    );
    stdout::writeln("  Files:");
    stdout::writeln("    • /Library/LaunchAgents (system-wide user agents)");
    if !home.is_empty() {
        stdout::writeln(&format!("    • {}/Library/LaunchAgents (user-specific agents)", home));
        stdout::writeln(&format!("    • {}/.config/angel/user (angel-managed user agents)", home));
    }
    stdout::writeln("");

    // GUI Domain
    stdout::writeln("GUI Domain");
    stdout::writeln("  Purpose: User-specific agents that require GUI session (Aqua)");
    stdout::writeln("  Use for: Services that need access to the graphical interface");
    stdout::writeln("  Files:");
    stdout::writeln("    • /System/Library/LaunchAgents (Apple GUI agents)");
    stdout::writeln("    • /System/Library/LaunchAngels (Apple GUI agents)");
    if !home.is_empty() {
        stdout::writeln(&format!("    • {}/.config/angel/gui (angel-managed GUI agents)", home));
    }
    stdout::writeln("");
}
