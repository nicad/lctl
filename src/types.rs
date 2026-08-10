use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Domain {
    System,
    User(u32),
    Gui(u32),
    Unknown,
}

impl Domain {
    pub fn from_plist(plist: &Plist, uid: u32, default: Domain) -> Self {
        if let Some(ref session_type) = plist.limit_load_to_session_type {
            let d = match session_type.as_str() {
                "Aqua" => Domain::Gui(uid),
                "Background" | "LoginWindow" => Domain::User(uid),
                "System" => Domain::System,
                _ => Domain::Unknown,
            };
            if d != Domain::Unknown { d } else { default }
        } else {
            default
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
    Angel,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    pub keep_alive: Option<bool>,

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
        }
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
