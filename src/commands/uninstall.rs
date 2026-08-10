use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::{err_msg, Error, Result};
use crate::launchctl;
use crate::output::{is_verbose, stderr, stdout};
use crate::parser::{self, ServiceItem};
use crate::types::Daemon;
use crate::utils;
use std::path::{Path, PathBuf};

pub(crate) fn run(angel: &Angel, args: &NameArgs) -> Result<()> {
    let daemon = super::resolve_daemon(angel, &args.name, args.exact)?;

    let source_path = get_source_path(daemon)?;

    if !utils::confirm(&format!(
        "Uninstall service `{}` at `{}`?",
        daemon.name,
        source_path.display()
    ))? {
        stdout::writeln("Uninstall cancelled.");
        return Ok(());
    }

    bootout_service(daemon);
    remove_plist_file(&source_path)?;
    remove_db_overrides(daemon)?;

    stdout::success(&format!("Uninstalled {}", daemon.name));
    Ok(())
}

fn get_source_path(daemon: &Daemon) -> Result<PathBuf> {
    if let Some(path) = &daemon.source_path {
        return Ok(path.clone());
    }

    let result = launchctl::print(&daemon.service_target())?;
    let parsed = parser::parse_launchctl_service(&result, daemon)?;
    match parsed.get("path") {
        Some(ServiceItem::Str(s)) => Ok(PathBuf::from(*s)),
        _ => Err(Error::InvalidInput(err_msg!(
            "Service does not have an installed plist file"
        ))),
    }
}

fn bootout_service(daemon: &Daemon) {
    match launchctl::bootout(daemon) {
        Ok(result) => {
            if result.success() {
                stdout::success(&format!("Unloaded service: {}", daemon.name));
            } else if is_verbose() {
                stderr::warn(&format!("Failed to unload service: {}", result.stderr));
            }
        }
        Err(e) => {
            if is_verbose() {
                stderr::warn(&format!("Failed to unload service: {}", e));
            }
        }
    }
}

fn remove_plist_file(source_path: &Path) -> Result<()> {
    let source_path_display = source_path.display().to_string();
    if source_path.exists() {
        std::fs::remove_file(source_path)?;
        stdout::success(&format!("Removed plist file: {}", source_path_display));
    } else if is_verbose() {
        stderr::warn(&format!("Plist file does not exist: {}", source_path_display));
    }
    Ok(())
}

fn remove_db_overrides(daemon: &Daemon) -> Result<()> {
    let db_overrides_file = PathBuf::from("/var/db/com.apple.xpc.launchd/disabled.plist");
    if !db_overrides_file.exists() {
        return Ok(());
    }

    let bytes = std::fs::read(&db_overrides_file)?;
    let disabled_services: std::collections::HashMap<String, bool> = plist::from_bytes(&bytes)?;

    if let Some(current_value) = disabled_services.get(&daemon.name) {
        if !utils::confirm(&format!(
            "Found `{}` in enable/disable override database with disabled = {}. Remove it?",
            daemon.name, current_value
        ))? {
            return Ok(());
        }
        let mut updated_services = disabled_services.clone();
        updated_services.remove(&daemon.name);
        let mut writer =
            std::fs::OpenOptions::new().write(true).truncate(true).open(&db_overrides_file)?;
        plist::to_writer_xml(&mut writer, &updated_services)?;
        stdout::success(&format!("Removed service from disabled.plist: {}", daemon.name));
    }

    Ok(())
}
