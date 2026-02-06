use crate::error::{Result, SystemError};
use crate::output;
use crate::styles::styles;
use crate::types::{Daemon, Domain};
use crossterm::style::Color;
use std::process::Command;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct LaunchctlResult {
    pub output: String,
    output_bytes: Vec<u8>,
    pub exit_code: Option<i32>,
    pub stderr: String,
}

impl LaunchctlResult {
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }

    pub fn output_bytes(&self) -> &[u8] {
        &self.output_bytes
    }

    /// Validates the result and returns the output string if successful.
    /// Returns an error if the command failed or if the output indicates a bad request.
    pub fn validated_output(&self, service_name: &str) -> crate::error::Result<&str> {
        if !self.success() {
            return Err(crate::error::SystemError::Launchctl(format!(
                "launchctl print failed for service: {}",
                service_name
            ))
            .into());
        }

        if self.output.starts_with("Bad request.") {
            return Err(crate::error::SystemError::Launchctl(format!(
                "Service not found: {}",
                service_name
            ))
            .into());
        }

        Ok(&self.output)
    }
}

static ROOT_STATUS: OnceLock<bool> = OnceLock::new();

fn is_root() -> bool {
    *ROOT_STATUS.get_or_init(|| unsafe { libc::geteuid() == 0 })
}

pub trait PrintTarget {
    fn target_str(&self) -> String;
}

impl PrintTarget for Daemon {
    fn target_str(&self) -> String {
        format!("{}/{}", self.domain_str(), self.name)
    }
}

impl PrintTarget for &str {
    fn target_str(&self) -> String {
        (*self).to_string()
    }
}

impl PrintTarget for String {
    fn target_str(&self) -> String {
        self.clone()
    }
}

pub fn bootstrap(daemon: &Daemon) -> Result<LaunchctlResult> {
    let path = daemon
        .source_path
        .as_ref()
        .ok_or_else(|| {
            SystemError::Launchctl("Cannot bootstrap daemon without source path".to_string())
        })?
        .to_str()
        .ok_or_else(|| SystemError::Launchctl("Invalid source path encoding".to_string()))?;
    let res = launchctl_exec(vec!["bootstrap", &daemon.domain_str(), path]);
    if res.as_ref().is_ok_and(|result| result.output.contains("Input/output error")) {
        match daemon.domain {
            Domain::User(_) => {
                output::stdout::hint(
                    "The user domain is for background sessions. If this isn't a background session, use the GUI domain.",
                );
            }
            _ => {}
        }
    }
    res
}

pub fn bootout(daemon: &Daemon) -> Result<LaunchctlResult> {
    launchctl_exec(vec!["bootout", &service_target(daemon)])
}

pub fn enable(daemon: &Daemon) -> Result<LaunchctlResult> {
    launchctl_exec(vec!["enable", &service_target(daemon)])
}

pub fn disable(daemon: &Daemon) -> Result<LaunchctlResult> {
    launchctl_exec(vec!["disable", &service_target(daemon)])
}

pub fn kickstart(daemon: &Daemon) -> Result<LaunchctlResult> {
    launchctl_exec(vec!["kickstart", &service_target(daemon)])
}

pub fn kickstart_kill(daemon: &Daemon) -> Result<LaunchctlResult> {
    launchctl_exec(vec!["kickstart", "-k", &service_target(daemon)])
}

pub fn kill(daemon: &Daemon, signal: &str) -> Result<LaunchctlResult> {
    launchctl_exec(vec!["kill", signal, &service_target(daemon)])
}

pub fn print<T: PrintTarget>(target: &T) -> Result<LaunchctlResult> {
    launchctl_exec(vec!["print", &target.target_str()])
}

fn launchctl_exec(mut args: Vec<&str>) -> Result<LaunchctlResult> {
    let mut cmd = if is_root() {
        args.insert(0, "launchctl");
        Command::new("sudo")
    } else {
        Command::new("launchctl")
    };

    if output::is_verbose() {
        let cmd_str = format!("{} {}", cmd.get_program().to_string_lossy(), args.join(" "));
        output::stdout::writelogln(styles::prefix(Color::Blue, "CMD"), styles::command(&cmd_str));
    }

    let output = cmd.args(&args).output().map_err(|e| {
        SystemError::Launchctl(format!("Failed to execute launchctl command: {}", e))
    })?;

    let exit_code = output.status.code();
    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(LaunchctlResult {
        output: stdout_str,
        output_bytes: output.stdout,
        exit_code,
        stderr: stderr_str,
    })
}

fn service_target(daemon: &Daemon) -> String {
    format!("{}/{}", daemon.domain_str(), daemon.name)
}
