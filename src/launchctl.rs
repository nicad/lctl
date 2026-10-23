use crate::error::{Error, Result, err_msg};
use crate::output;
use crate::styles;
use crate::sudo;
use crate::types::{Daemon, Domain};
use crossterm::style::Color;
use std::process::Command;

const LAUNCHCTL: &str = "/bin/launchctl";

#[derive(Debug)]
pub struct LaunchctlOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl LaunchctlOutput {
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
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

    fn looks_like_needs_root(&self) -> bool {
        sudo::looks_like_needs_root(&self.stdout, &self.stderr)
    }

    /// What to do about this failed call on `daemon`, from launchd's error message.
    pub fn failure_hint(&self, daemon: &Daemon) -> Option<String> {
        let name = &daemon.name;
        let has = |message: &str| self.stdout.contains(message) || self.stderr.contains(message);

        if has("System Integrity Protection") {
            return Some(
                "protected by System Integrity Protection; it can't be changed".to_string(),
            );
        }
        if has("Service is disabled") {
            return Some(format!("run `lctl enable {name}`"));
        }
        if has("bad ownership/permissions") {
            return Some(format!("`lctl doctor {name}` shows what to fix"));
        }
        if has("Could not find service") || has("Could not find specified service") {
            return Some(format!(
                "not loaded in {}; `lctl start {name}` loads and starts it",
                daemon.domain
            ));
        }
        if has("No such process") {
            return Some(format!("not loaded in {}", daemon.domain));
        }
        if has("Invalid signal") {
            return Some(
                "use a signal name like TERM, HUP or KILL, or a number (`kill -l` lists them)"
                    .to_string(),
            );
        }
        if has("Operation already in progress") {
            return Some(format!("already loaded; `lctl reload {name}` reloads it"));
        }
        // launchd's catch-all for a refused load; lctl checks disabled before loading
        if has("Input/output error") {
            return Some(match daemon.domain {
                _ if daemon.loaded => format!("already loaded; `lctl reload {name}` reloads it"),
                Domain::User(_) => format!(
                    "launchd refused the plist: `lctl doctor {name}` checks it. The user domain is \
                     for background sessions; agents for your login session belong in gui"
                ),
                _ => format!("launchd refused the plist: `lctl doctor {name}` checks it"),
            });
        }
        if self.looks_like_needs_root() {
            return Some(
                "needs root: run it from a terminal (lctl asks for sudo) or with sudo".to_string(),
            );
        }
        None
    }
}

pub fn bootstrap(daemon: &Daemon) -> Result<LaunchctlOutput> {
    let path = daemon
        .require_source_path()?
        .to_str()
        .ok_or_else(|| Error::InvalidInput(err_msg!("Invalid source path encoding")))?;
    let domain = daemon.domain.to_string();
    launchctl_exec(&["bootstrap", &domain, path])
}

pub fn bootout(daemon: &Daemon) -> Result<LaunchctlOutput> {
    launchctl_exec(&["bootout", &daemon.service_target()])
}

pub fn enable(daemon: &Daemon) -> Result<LaunchctlOutput> {
    launchctl_exec(&["enable", &daemon.service_target()])
}

pub fn disable(daemon: &Daemon) -> Result<LaunchctlOutput> {
    launchctl_exec(&["disable", &daemon.service_target()])
}

pub fn kickstart(daemon: &Daemon) -> Result<LaunchctlOutput> {
    launchctl_exec(&["kickstart", &daemon.service_target()])
}

pub fn kickstart_kill(daemon: &Daemon) -> Result<LaunchctlOutput> {
    launchctl_exec(&["kickstart", "-k", &daemon.service_target()])
}

pub fn kill(daemon: &Daemon, signal: &str) -> Result<LaunchctlOutput> {
    launchctl_exec(&["kill", signal, &daemon.service_target()])
}

pub fn print(target: &str) -> Result<LaunchctlOutput> {
    launchctl_exec(&["print", target])
}

fn launchctl_exec(args: &[&str]) -> Result<LaunchctlOutput> {
    let first = run_launchctl(args, false)?;

    if first.success() || !first.looks_like_needs_root() || sudo::is_root() || !sudo::stdin_is_tty()
    {
        return Ok(first);
    }

    // TTY + need-root: suppress first failure, prompt, retry with sudo -n.
    if output::is_verbose() {
        output::stdout::writelogln(
            styles::prefix(Color::Yellow, "HINT"),
            "launchctl needs root; prompting for sudo then retrying",
        );
    }

    sudo::ensure_elevated()?;
    run_launchctl(args, true)
}

fn run_launchctl(args: &[&str], elevated: bool) -> Result<LaunchctlOutput> {
    let output = if elevated {
        sudo::run_sudo_n(&[&[LAUNCHCTL][..], args].concat())?
    } else {
        if output::is_verbose() {
            let cmd_str = format!("{LAUNCHCTL} {}", args.join(" "));
            output::stdout::writelogln(
                styles::prefix(Color::Blue, "CMD"),
                styles::command(&cmd_str),
            );
        }
        Command::new(LAUNCHCTL).args(args).output()?
    };

    Ok(LaunchctlOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code(),
    })
}
