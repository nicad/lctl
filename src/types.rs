use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    System,
    User(u32),
    Gui(u32),
    Unknown,
}

impl Domain {
    pub fn from_plist(plist: &Plist, uid: u32, default: Domain) -> Self {
        match plist.limit_load_to_session_type.as_deref() {
            Some("Aqua") => Domain::Gui(uid),
            Some("Background") | Some("LoginWindow") => Domain::User(uid),
            Some("System") => Domain::System,
            _ => default,
        }
    }

    /// The user a `gui`/`user` domain belongs to; services there run as that user.
    pub fn user_uid(&self) -> Option<u32> {
        match self {
            Domain::Gui(uid) | Domain::User(uid) => Some(*uid),
            Domain::System | Domain::Unknown => None,
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Domain::System => write!(f, "system"),
            Domain::User(uid) => write!(f, "user/{}", uid),
            Domain::Gui(uid) => write!(f, "gui/{}", uid),
            Domain::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForWhom {
    User,
    Apple,
    ThirdParty,
    Lctl,
}

/// launchd KeepAlive may be a bool or a dictionary of conditions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum KeepAlive {
    Boolean(bool),
    Dictionary(std::collections::HashMap<String, plist::Value>),
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Plist {
    #[serde(rename = "Label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(rename = "Program", skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,

    #[serde(rename = "ProgramArguments", skip_serializing_if = "Option::is_none")]
    pub program_arguments: Option<Vec<String>>,

    #[serde(rename = "RunAtLoad", skip_serializing_if = "Option::is_none")]
    pub run_at_load: Option<bool>,

    #[serde(rename = "KeepAlive", skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<KeepAlive>,

    #[serde(rename = "WorkingDirectory", skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    #[serde(rename = "StandardOutPath", skip_serializing_if = "Option::is_none")]
    pub standard_out_path: Option<String>,

    #[serde(rename = "StandardErrorPath", skip_serializing_if = "Option::is_none")]
    pub standard_error_path: Option<String>,

    #[serde(rename = "EnvironmentVariables", skip_serializing_if = "Option::is_none")]
    pub environment_variables: Option<std::collections::HashMap<String, String>>,

    #[serde(rename = "StartInterval", skip_serializing_if = "Option::is_none")]
    pub start_interval: Option<i32>,

    #[serde(rename = "StartOnMount", skip_serializing_if = "Option::is_none")]
    pub start_on_mount: Option<bool>,

    #[serde(rename = "ThrottleInterval", skip_serializing_if = "Option::is_none")]
    pub throttle_interval: Option<i32>,

    #[serde(rename = "ProcessType", skip_serializing_if = "Option::is_none")]
    pub process_type: Option<String>,

    #[serde(rename = "SessionCreate", skip_serializing_if = "Option::is_none")]
    pub session_create: Option<bool>,

    #[serde(rename = "LaunchOnlyOnce", skip_serializing_if = "Option::is_none")]
    pub launch_only_once: Option<bool>,

    #[serde(rename = "LimitLoadToSessionType", skip_serializing_if = "Option::is_none")]
    pub limit_load_to_session_type: Option<String>,

    // Launch triggers kept as raw values: only their presence (and the calendar fields) matter,
    // and a typed shape would make an unusual but valid plist fail to parse.
    #[serde(rename = "StartCalendarInterval", skip_serializing_if = "Option::is_none")]
    pub start_calendar_interval: Option<plist::Value>,

    #[serde(rename = "WatchPaths", skip_serializing_if = "Option::is_none")]
    pub watch_paths: Option<plist::Value>,

    #[serde(rename = "QueueDirectories", skip_serializing_if = "Option::is_none")]
    pub queue_directories: Option<plist::Value>,

    #[serde(rename = "Sockets", skip_serializing_if = "Option::is_none")]
    pub sockets: Option<plist::Value>,

    #[serde(rename = "MachServices", skip_serializing_if = "Option::is_none")]
    pub mach_services: Option<plist::Value>,
}

impl Plist {
    /// The executable launchd runs: `Program`, else `ProgramArguments[0]`.
    pub fn executable(&self) -> Option<&str> {
        self.program
            .as_deref()
            .or_else(|| self.program_arguments.as_ref()?.first().map(String::as_str))
    }
}

#[derive(Debug, Clone)]
pub struct Daemon {
    pub name: String,
    pub source_path: Option<PathBuf>,
    pub domain: Domain,
    pub for_use_by: ForWhom,
    pub plist: Option<Plist>,
    pub pid: Option<u32>,
    pub last_exit_code: Option<String>,
    /// Bootstrapped in launchd (listed by `launchctl print <domain>`).
    pub loaded: bool,
    /// launchd's persistent disabled flag is set for this label in `domain`.
    pub disabled: bool,
}

/// Service state as shown by `list` and `status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Running,
    Idle,
    /// Loaded, not running, and the last exit code was non-zero.
    Failed(i32),
    NotLoaded,
    Disabled,
}

impl ServiceState {
    /// Display order for summaries.
    pub const NAMES: [&'static str; 5] = ["running", "idle", "failed", "not loaded", "disabled"];

    /// The state without details (the exit code of `Failed`).
    pub fn name(&self) -> &'static str {
        match self {
            ServiceState::Running => "running",
            ServiceState::Idle => "idle",
            ServiceState::Failed(_) => "failed",
            ServiceState::NotLoaded => "not loaded",
            ServiceState::Disabled => "disabled",
        }
    }
}

impl fmt::Display for ServiceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceState::Failed(code) => write!(f, "failed (exit {code})"),
            state => f.write_str(state.name()),
        }
    }
}

impl Daemon {
    pub fn from_plist(
        plist: Plist,
        path: Option<PathBuf>,
        default_domain: Domain,
        for_use_by: ForWhom,
        uid: u32,
    ) -> Self {
        let name = plist.label.clone().unwrap_or_else(|| {
            path.as_ref()
                .and_then(|p| p.file_stem())
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        });
        let domain = Domain::from_plist(&plist, uid, default_domain);
        Self {
            name,
            source_path: path,
            domain,
            for_use_by,
            plist: Some(plist),
            pid: None,
            last_exit_code: None,
            loaded: false,
            disabled: false,
        }
    }

    pub fn state(&self) -> ServiceState {
        if self.pid.is_some() {
            return ServiceState::Running;
        }
        if self.loaded {
            // exit code is `-` (never exited), a number, or a marker like `(jt)`
            return match self.last_exit_code.as_deref().and_then(|code| code.parse().ok()) {
                Some(code) if code != 0 => ServiceState::Failed(code),
                _ => ServiceState::Idle,
            };
        }
        if self.disabled { ServiceState::Disabled } else { ServiceState::NotLoaded }
    }

    pub fn service_target(&self) -> String {
        format!("{}/{}", self.domain, self.name)
    }

    pub fn require_source_path(&self) -> crate::error::Result<&std::path::Path> {
        use crate::error::{Error, err_msg};
        self.source_path.as_deref().ok_or_else(|| {
            Error::InvalidInput(err_msg!("`{}` has no plist path on disk", self.name))
        })
    }
}
