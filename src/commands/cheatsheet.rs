use crate::angel::{angel_domain_dirs, get_angel_dir};
use crate::cli::CheatsheetSubcommand;
use crate::output::stdout;
use crate::styles;
use crate::types::Domain;

pub fn run(subcommand: &CheatsheetSubcommand) {
    match subcommand {
        CheatsheetSubcommand::Domains => print_domains(),
    }
}

fn print_domains() {
    let _ = crate::utils::init_home();
    let angel_dir = get_angel_dir().unwrap_or_else(|_| {
        crate::utils::home().join(".config").join("angel")
    });
    let uid = nix::unistd::getuid().as_raw();
    let dirs = angel_domain_dirs(&angel_dir, uid);
    let angel_system = dirs
        .iter()
        .find(|(_, d, _)| matches!(d, Domain::System))
        .map(|(p, _, _)| styles::display_source_path(p, false))
        .unwrap_or_default();
    let angel_user = dirs
        .iter()
        .find(|(_, d, _)| matches!(d, Domain::User(_)))
        .map(|(p, _, _)| styles::display_source_path(p, false))
        .unwrap_or_default();
    let angel_gui = dirs
        .iter()
        .find(|(_, d, _)| matches!(d, Domain::Gui(_)))
        .map(|(p, _, _)| styles::display_source_path(p, false))
        .unwrap_or_default();

    stdout::writeln("Launchd Domains");
    stdout::writeln("");

    // System Domain
    stdout::writeln("System Domain");
    stdout::writeln("  Purpose: System-wide daemons that run as root, regardless of user login");
    stdout::writeln("  Use for: Services that need to run at system boot, before user login");
    stdout::writeln("  Files:");
    stdout::writeln("    • /System/Library/LaunchDaemons (Apple system daemons)");
    stdout::writeln("    • /Library/LaunchDaemons (third-party system daemons)");
    stdout::writeln(&format!("    • {} (angel-managed system daemons)", angel_system));
    stdout::writeln("");

    // User Domain
    stdout::writeln("User Domain");
    stdout::writeln("  Purpose: User-specific agents that run in the background, not tied to GUI");
    stdout::writeln(
        "  Use for: Services that should run when user logs in, but don't need GUI access",
    );
    stdout::writeln("  Files:");
    stdout::writeln("    • /Library/LaunchAgents (system-wide user agents)");
    stdout::writeln("    • ~/Library/LaunchAgents (user-specific agents)");
    stdout::writeln(&format!("    • {} (angel-managed user agents)", angel_user));
    stdout::writeln("");

    // GUI Domain
    stdout::writeln("GUI Domain");
    stdout::writeln("  Purpose: User-specific agents that require GUI session (Aqua)");
    stdout::writeln("  Use for: Services that need access to the graphical interface");
    stdout::writeln("  Files:");
    stdout::writeln("    • /System/Library/LaunchAgents (Apple GUI agents)");
    stdout::writeln("    • /System/Library/LaunchAngels (Apple GUI agents)");
    stdout::writeln(&format!("    • {} (angel-managed GUI agents)", angel_gui));
    stdout::writeln("");
}
