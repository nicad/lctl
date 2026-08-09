use crate::error::{Error, Result, err_msg};
use crate::output;
use crate::styles::styles;
use crate::types::{Daemon, Domain};
use crossterm::style::Color;
use std::process::Command;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct LaunchctlOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    stdout_bytes: Vec<u8>,
}

impl LaunchctlOutput {
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }

    pub fn stdout_bytes(&self) -> &[u8] {
        &self.stdout_bytes
    }

    /// Convert a non-zero launchctl exit into [`Error::Launchctl`].
    pub fn require_ok(&self, action: &str) -> Result<&Self> {
        if self.success() {
            Ok(self)
        } else {
            let detail = if self.stderr.is_empty() {
                format!("exit code {:?}", self.exit_code)
            } else {
                self.stderr.clone()
            };
            Err(Error::launchctl(action, detail))
        }
    }

    /// Require success, reject launchctl "Bad request." as not found, return stdout.
    pub fn require_print_output(&self, service_name: &str) -> Result<&str> {
        if !self.success() {
            return Err(Error::launchctl(
                format!("print {}", service_name),
                format!("launchctl print failed for service: {}", service_name),
            ));
        }

        if self.stdout.starts_with("Bad request.") {
            return Err(Error::NotFound(err_msg!(service_name)));
        }

        Ok(&self.stdout)
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

pub fn bootstrap(daemon: &Daemon) -> Result<LaunchctlOutput> {
    let path = daemon
        .source_path
        .as_ref()
        .ok_or_else(|| Error::InvalidInput(err_msg!("Cannot bootstrap daemon without source path")))?
        .to_str()
        .ok_or_else(|| Error::InvalidInput(err_msg!("Invalid source path encoding")))?;
    let res = launchctl_exec(vec!["bootstrap", &daemon.domain_str(), path]);
    if res.as_ref().is_ok_and(|result| result.stdout.contains("Input/output error")) {
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

pub fn bootout(daemon: &Daemon) -> Result<LaunchctlOutput> {
    launchctl_exec(vec!["bootout", &service_target(daemon)])
}

pub fn enable(daemon: &Daemon) -> Result<LaunchctlOutput> {
    launchctl_exec(vec!["enable", &service_target(daemon)])
}

pub fn disable(daemon: &Daemon) -> Result<LaunchctlOutput> {
    launchctl_exec(vec!["disable", &service_target(daemon)])
}

pub fn kickstart(daemon: &Daemon) -> Result<LaunchctlOutput> {
    launchctl_exec(vec!["kickstart", &service_target(daemon)])
}

pub fn kickstart_kill(daemon: &Daemon) -> Result<LaunchctlOutput> {
    launchctl_exec(vec!["kickstart", "-k", &service_target(daemon)])
}

pub fn kill(daemon: &Daemon, signal: &str) -> Result<LaunchctlOutput> {
    launchctl_exec(vec!["kill", signal, &service_target(daemon)])
}

pub fn print<T: PrintTarget>(target: &T) -> Result<LaunchctlOutput> {
    launchctl_exec(vec!["print", &target.target_str()])
}

fn launchctl_exec(mut args: Vec<&str>) -> Result<LaunchctlOutput> {
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
        Error::unexpected(format!("Failed to execute launchctl command: {}", e))
    })?;

    let exit_code = output.status.code();
    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(LaunchctlOutput {
        stdout: stdout_str,
        stderr: stderr_str,
        exit_code,
        stdout_bytes: output.stdout,
    })
}

fn service_target(daemon: &Daemon) -> String {
    format!("{}/{}", daemon.domain_str(), daemon.name)
}
