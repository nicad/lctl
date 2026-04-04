use crate::angel::Angel;
use crate::cli::LogsArgs;
use crate::error::Result;
use crate::output::stdout;
use crate::styles::styles;
use crossterm::style::Color;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

pub fn run(angel: &Angel, args: &LogsArgs) -> Result<()> {
    let daemon = angel.daemons.get_match(&args.name, args.exact)?;

    stdout::writeln(&styles::prefix(Color::Blue, &daemon.name));
    stdout::writeln("");

    let mut found_any_logs = false;

    // Check plist for StandardOutPath and StandardErrorPath
    if let Some(plist) = &daemon.plist {
        // Show stdout logs
        if let Some(ref stdout_path) = plist.standard_out_path {
            if Path::new(stdout_path).exists() {
                found_any_logs = true;
                stdout::writeln(&styles::prefix(
                    Color::Yellow,
                    &format!("stdout: {}", stdout_path),
                ));
                if let Ok(content) = read_log_file(stdout_path) {
                    if !content.is_empty() {
                        stdout::writeln(&content);
                    } else {
                        stdout::writeln("  (empty)");
                    }
                } else {
                    stdout::writeln("  (cannot read - may require sudo)");
                }
                stdout::writeln("");
            }
        }

        // Show stderr logs
        if let Some(ref stderr_path) = plist.standard_error_path {
            if Path::new(stderr_path).exists() {
                found_any_logs = true;
                stdout::writeln(&styles::prefix(
                    Color::Yellow,
                    &format!("stderr: {}", stderr_path),
                ));
                if let Ok(content) = read_log_file(stderr_path) {
                    if !content.is_empty() {
                        stdout::writeln(&content);
                    } else {
                        stdout::writeln("  (empty)");
                    }
                } else {
                    stdout::writeln("  (cannot read - may require sudo)");
                }
                stdout::writeln("");
            }
        }
    }

    // Always show unified logs (in addition to plist logs)
    let process_name = extract_process_name(&daemon);
    stdout::writeln(&styles::prefix(
        Color::Yellow,
        &format!("unified logs (last {}) - process: {}", args.time_frame, process_name),
    ));
    if let Ok(logs) = get_unified_logs(&process_name, &args.time_frame) {
        if !logs.is_empty() {
            stdout::writeln(&logs);
            found_any_logs = true;
        } else {
            stdout::writeln("  (no logs found)");
        }
    } else {
        stdout::writeln("  (cannot read unified logs)");
    }

    if !found_any_logs {
        stdout::writeln("");
        stdout::writeln("  (no logs available from any source)");
    }

    Ok(())
}

fn read_log_file(path: &str) -> Result<String> {
    let mut file = fs::File::open(path).map_err(crate::error::AngelError::Io)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(crate::error::AngelError::Io)?;

    // Show last 50 lines if file is large
    let lines: Vec<&str> = contents.lines().collect();
    if lines.len() > 50 { Ok(lines[lines.len() - 50..].join("\n")) } else { Ok(contents) }
}

fn extract_process_name(daemon: &crate::types::Daemon) -> String {
    if let Some(plist) = &daemon.plist {
        // Try to get process name from Program
        if let Some(ref program) = plist.program {
            if let Some(name) = std::path::Path::new(program).file_name() {
                if let Some(name_str) = name.to_str() {
                    return name_str.to_string();
                }
            }
        }
        // Try to get from ProgramArguments (first argument is usually the program)
        if let Some(ref args) = plist.program_arguments {
            if let Some(first_arg) = args.first() {
                if let Some(name) = std::path::Path::new(first_arg).file_name() {
                    if let Some(name_str) = name.to_str() {
                        return name_str.to_string();
                    }
                }
            }
        }
    }
    // Fallback to service name
    daemon.name.clone()
}

fn get_unified_logs(process_name: &str, time_frame: &str) -> Result<String> {
    let output = Command::new("log")
        .args(&[
            "show",
            "--predicate",
            &format!("process == \"{}\"", process_name),
            "--last",
            time_frame,
            "--style",
            "compact",
        ])
        .output()
        .map_err(crate::error::AngelError::Io)?;

    if output.status.success() {
        let logs = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = logs.lines().collect();
        // Show last 50 lines
        if lines.len() > 50 {
            Ok(lines[lines.len() - 50..].join("\n"))
        } else {
            Ok(logs.to_string())
        }
    } else {
        Ok(String::new())
    }
}
