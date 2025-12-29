use crate::angel::Angel;
use crate::cli::StatusArgs;
use crate::error::Result;
use crate::launchctl;
use crate::output::{is_verbose, stdout};
use crate::styles::styles;
use crossterm::style::Color;

pub fn run(angel: &Angel, args: &StatusArgs) -> Result<()> {
    let daemon = angel.daemons.get_match(&args.name, args.exact)?;
    let result = launchctl::print(daemon)?;

    let status = extract_status(&result.output);
    let color = if result.success() { None } else { Some(Color::Red) };

    stdout::writeln(&styles::prefix(Color::Blue, &daemon.name));
    stdout::writeln(&styles::format_status_dot(&status, color));

    let mut table = styles::create_table();
    table.add_row(vec!["Domain:".to_string(), daemon.domain_str()]);
    table.add_row(vec!["Source:".to_string(), styles::display_path(daemon, true)]);

    if is_verbose() {
        // Add plist fields if available
        if let Some(plist) = &daemon.plist {
            if let Some(program) = &plist.program {
                table.add_row(vec!["Program:".to_string(), program.clone()]);
            }
            if let Some(program_arguments) = &plist.program_arguments {
                table.add_row(vec!["ProgramArguments:".to_string(), program_arguments.join(" ")]);
            }
        }
    }

    stdout::writeln(&table);
    Ok(())
}

fn extract_status(output: &str) -> String {
    // Simple extraction - look for common status indicators
    match () {
        _ if output.contains("state = running") => "running".to_string(),
        _ if output.contains("state = stopped") => "stopped".to_string(),
        _ if output.contains("state = launched") => "launched".to_string(),
        _ if output.contains("state = exited") => "exited".to_string(),
        _ if output.contains("state = not running") => "not running".to_string(),
        _ => "unknown".to_string(),
    }
}
