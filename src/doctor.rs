use crate::describe;
use crate::parser::{self, ParsedService};
use crate::types::{Daemon, Plist, ServiceState};
use nix::unistd::{AccessFlags, access};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone)]
pub struct Check {
    pub severity: Severity,
    pub message: String,
    /// Command that fixes or investigates the problem.
    pub fix: Option<String>,
}

impl Check {
    pub fn ok(message: impl Into<String>) -> Self {
        Self { severity: Severity::Ok, message: message.into(), fix: None }
    }

    fn warn(message: impl Into<String>, fix: Option<String>) -> Self {
        Self { severity: Severity::Warn, message: message.into(), fix }
    }

    fn fail(message: impl Into<String>, fix: Option<String>) -> Self {
        Self { severity: Severity::Fail, message: message.into(), fix }
    }

    pub fn is_problem(&self) -> bool {
        self.severity != Severity::Ok
    }
}

/// launchd's PATH for services that don't set their own.
const DEFAULT_PATH: &str = "/usr/bin:/bin:/usr/sbin:/sbin";

/// Whether the current state needs attention: failed, disabled or not loaded.
pub fn check_state(daemon: &Daemon) -> Check {
    let name = &daemon.name;
    match daemon.state() {
        ServiceState::Disabled => {
            Check::warn("disabled: launchd won't load it", Some(format!("lctl enable {name}")))
        }
        ServiceState::NotLoaded if in_launch_directory(daemon) => Check::warn(
            "not loaded (launchd loads it at the next login/boot)",
            Some(format!("lctl start {name}")),
        ),
        ServiceState::NotLoaded => Check::warn(
            "not installed: the plist is outside the directories launchd reads",
            Some(format!("lctl install {name}")),
        ),
        ServiceState::Failed(code) => {
            Check::warn(format!("last run exited with {code}"), Some(format!("lctl logs {name}")))
        }
        state => Check::ok(state.to_string()),
    }
}

/// Check the state, the plist and what it points to. `print` is the parsed `launchctl print`,
/// when loaded.
pub fn check(daemon: &Daemon, print: Option<&ParsedService>) -> Vec<Check> {
    // launchd ignores a plist without a Label (e.g. an emptied stub); nothing else applies
    if daemon.plist.as_ref().is_some_and(|plist| plist.label.is_none()) {
        return vec![Check::fail("no Label: launchd ignores this file", None)];
    }
    let mut checks = vec![check_state(daemon)];
    let (Some(plist), Some(path)) = (&daemon.plist, &daemon.source_path) else {
        checks.push(check_missing_plist(daemon, print));
        return checks;
    };
    check_plist_file(daemon, plist, path, &mut checks);
    checks.push(check_program(plist));
    check_directories(daemon, plist, &mut checks);
    check_triggers(daemon, plist, &mut checks);
    if let Some(print) = print {
        check_loaded_matches_disk(daemon, plist, print, &mut checks);
    }
    checks
}

fn check_missing_plist(daemon: &Daemon, print: Option<&ParsedService>) -> Check {
    match print.and_then(|print| parser::str_field(print, "path")) {
        Some(path) if !Path::new(path).exists() => Check::fail(
            format!("loaded from {path}, which no longer exists"),
            Some(format!("lctl stop {}", daemon.name)),
        ),
        Some(path) => Check::ok(format!("plist {path} (outside the directories lctl reads)")),
        None => Check::ok("no plist: registered by an app or by launchd itself"),
    }
}

fn check_plist_file(daemon: &Daemon, plist: &Plist, path: &Path, checks: &mut Vec<Check>) {
    let shown = path.display();
    if path.file_stem().and_then(|stem| stem.to_str()) != Some(daemon.name.as_str()) {
        checks.push(Check::warn(
            format!("file name doesn't match its Label (expected {}.plist)", daemon.name),
            None,
        ));
    }

    match std::fs::metadata(path) {
        Err(e) => checks.push(Check::fail(format!("can't read {shown}: {e}"), None)),
        Ok(meta) => {
            // launchd ignores plists with the wrong owner or that others can write to
            let (owner_ok, owner, fix) = match daemon.domain.user_uid() {
                Some(uid) if !path.starts_with("/Library") => (
                    meta.uid() == uid || meta.uid() == 0,
                    format!("uid {uid}"),
                    format!("sudo chown {uid} {shown} && chmod 644 {shown}"),
                ),
                _ => (
                    meta.uid() == 0,
                    "root".to_string(),
                    format!("sudo chown root:wheel {shown} && sudo chmod 644 {shown}"),
                ),
            };
            if !owner_ok || meta.mode() & 0o022 != 0 {
                checks.push(Check::fail(
                    format!(
                        "launchd needs it owned by {owner} and not writable by group/others \
                         (owner uid {}, mode {:o})",
                        meta.uid(),
                        meta.mode() & 0o777
                    ),
                    Some(fix),
                ));
            } else {
                checks.push(Check::ok("plist owner and permissions"));
            }
        }
    }

    let directory = path.parent().and_then(|dir| dir.file_name()).and_then(|name| name.to_str());
    match (directory, plist.limit_load_to_session_type.as_deref()) {
        (Some("LaunchDaemons"), Some(session)) if session != "System" => {
            checks.push(Check::warn(
                format!("LimitLoadToSessionType {session} in LaunchDaemons, which load as system"),
                None,
            ));
        }
        (Some("LaunchAgents"), Some("System")) => checks.push(Check::warn(
            "LimitLoadToSessionType System in LaunchAgents, which load per user",
            None,
        )),
        _ => {}
    }
}

fn check_program(plist: &Plist) -> Check {
    let Some(program) = plist.executable() else {
        return Check::fail("no Program or ProgramArguments", None);
    };
    if program.starts_with('/') {
        let path = Path::new(program);
        return if is_executable(path) {
            Check::ok(format!("program {program}"))
        } else if path.exists() {
            Check::fail(format!("program is not an executable file: {program}"), None)
        } else {
            Check::fail(format!("program not found: {program}"), None)
        };
    }
    if program.contains('/') {
        return Check::ok(format!("program {program} (relative path, not checked)"));
    }
    let search_path = plist
        .environment_variables
        .as_ref()
        .and_then(|env| env.get("PATH"))
        .map_or(DEFAULT_PATH, String::as_str);
    if search_path.split(':').any(|dir| is_executable(&Path::new(dir).join(program))) {
        Check::ok(format!("program {program} (found in PATH)"))
    } else {
        Check::fail(format!("program {program} not found in launchd's PATH ({search_path})"), None)
    }
}

fn check_directories(daemon: &Daemon, plist: &Plist, checks: &mut Vec<Check>) {
    if let Some(dir) = &plist.working_directory {
        if !Path::new(dir).is_dir() {
            checks.push(Check::fail(format!("WorkingDirectory doesn't exist: {dir}"), None));
        }
    }

    let mut logs = vec![&plist.standard_out_path, &plist.standard_error_path];
    logs.dedup();
    for log in logs.into_iter().flatten() {
        let Some(dir) = Path::new(log).parent() else { continue };
        if !dir.is_dir() {
            checks.push(Check::warn(
                format!("log directory doesn't exist, so output is lost: {}", dir.display()),
                Some(mkdir_fix(daemon, dir)),
            ));
            continue;
        }
        // agents run as you, so your own access is what counts
        let runs_as_you = daemon.domain.user_uid().is_some();
        let target = if Path::new(log).exists() { Path::new(log) } else { dir };
        if runs_as_you && access(target, AccessFlags::W_OK).is_err() {
            checks.push(Check::warn(format!("log not writable: {}", target.display()), None));
        }
    }
}

/// Agents run as you, so a directory created with sudo must be handed back to you.
fn mkdir_fix(daemon: &Daemon, dir: &Path) -> String {
    let shown = dir.display();
    let parent_writable = dir
        .ancestors()
        .find(|ancestor| ancestor.is_dir())
        .is_some_and(|ancestor| access(ancestor, AccessFlags::W_OK).is_ok());
    match daemon.domain.user_uid() {
        _ if parent_writable => format!("mkdir -p {shown}"),
        Some(uid) => format!("sudo mkdir -p {shown} && sudo chown {uid} {shown}"),
        None => format!("sudo mkdir -p {shown}"),
    }
}

fn check_triggers(daemon: &Daemon, plist: &Plist, checks: &mut Vec<Check>) {
    let triggers = describe::triggers(plist);
    if triggers.is_empty() {
        checks.push(Check::warn(
            "no launch trigger (RunAtLoad, KeepAlive, StartInterval, …): it only runs when started",
            Some(format!("lctl start {}", daemon.name)),
        ));
    } else {
        checks.push(Check::ok(format!("runs {}", triggers.join(", "))));
    }
    if let Some(seconds) = plist.start_interval.filter(|&seconds| seconds < 10) {
        checks.push(Check::warn(format!("StartInterval of {seconds}s keeps the CPU busy"), None));
    }
}

/// launchd keeps the plist it loaded; edits on disk need a reload.
fn check_loaded_matches_disk(
    daemon: &Daemon,
    plist: &Plist,
    print: &ParsedService,
    checks: &mut Vec<Check>,
) {
    let Some(loaded_program) = parser::str_field(print, "program") else { return };
    let loaded_arguments = parser::list_field(print, "arguments");
    let disk_arguments =
        plist.program_arguments.as_ref().map(|args| args.iter().map(String::as_str).collect());
    if plist.executable() != Some(loaded_program) || loaded_arguments != disk_arguments {
        checks.push(Check::warn(
            "plist changed on disk since it was loaded (program or arguments differ)",
            Some(format!("lctl reload {}", daemon.name)),
        ));
    }
}

fn is_executable(path: &Path) -> bool {
    path.metadata().is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

fn in_launch_directory(daemon: &Daemon) -> bool {
    daemon.source_path.as_deref().and_then(Path::parent).is_some_and(|dir| {
        dir.ends_with("Library/LaunchAgents") || dir.ends_with("Library/LaunchDaemons")
    })
}
