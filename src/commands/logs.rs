use crate::angel::Angel;
use crate::cli::LogsArgs;
use crate::error::Result;
use crate::output::stdout;
use crate::styles;
use crossterm::style::Color;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

pub fn run(angel: &Angel, args: &LogsArgs) -> Result<()> {
    let daemon = angel.daemons.get_match(&args.target.name, args.target.exact)?;

    stdout::writeln(&styles::prefix(Color::Blue, &daemon.name));
    stdout::writeln("");

    let mut found_any_logs = false;

    if let Some(plist) = &daemon.plist {
        if let Some(ref stdout_path) = plist.standard_out_path {
            found_any_logs |= emit_log_section("stdout", stdout_path);
        }
        if let Some(ref stderr_path) = plist.standard_error_path {
            found_any_logs |= emit_log_section("stderr", stderr_path);
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

fn emit_log_section(label: &str, path: &str) -> bool {
    if !Path::new(path).exists() {
        return false;
    }
    stdout::writeln(&styles::prefix(Color::Yellow, &format!("{}: {}", label, path)));
    match read_log_file(path) {
        Ok(content) if !content.is_empty() => stdout::writeln(&content),
        Ok(_) => stdout::writeln("  (empty)"),
        Err(_) => stdout::writeln("  (cannot read - may require sudo)"),
    }
    stdout::writeln("");
    true
}

fn last_n_lines(contents: &str, n: usize) -> String {
    let lines: Vec<&str> = contents.lines().collect();
    if lines.len() > n { lines[lines.len() - n..].join("\n") } else { contents.to_string() }
}

fn read_log_file(path: &str) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(last_n_lines(&contents, 50))
}

fn extract_process_name(daemon: &crate::types::Daemon) -> String {
    if let Some(plist) = &daemon.plist {
        if let Some(ref program) = plist.program {
            if let Some(name) = Path::new(program).file_name().and_then(|n| n.to_str()) {
                return name.to_string();
            }
        }
        if let Some(ref args) = plist.program_arguments {
            if let Some(first_arg) = args.first() {
                if let Some(name) = Path::new(first_arg).file_name().and_then(|n| n.to_str()) {
                    return name.to_string();
                }
            }
        }
    }
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
        .output()?;

    if output.status.success() {
        let logs = String::from_utf8_lossy(&output.stdout);
        Ok(last_n_lines(&logs, 50))
    } else {
        Ok(String::new())
    }
}
