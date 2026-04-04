use crate::angel::Angel;
use crate::cli::NameArgs;
use crate::error::{AngelError, Result};
use crate::launchctl;
use crate::output::{is_verbose, stderr, stdout};
use crate::parser::{Parser, ServiceItem};
use crate::types::Daemon;
use std::path::PathBuf;

pub fn run(angel: &Angel, args: &NameArgs) -> Result<()> {
    let daemon = angel.daemons.get_match(&args.name, args.exact)?;

    let source_path = get_source_path(daemon)?;

    if !confirm_uninstall(daemon, &source_path)? {
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

    let result = launchctl::print(daemon)?;
    let parsed = Parser::parse_print_service(&result, daemon)?;
    match parsed.get("path") {
        Some(ServiceItem::Str(s)) => Ok(PathBuf::from(*s)),
        _ => Err(AngelError::InvalidArgument(
            "Service does not have an installed plist file".to_string(),
        )),
    }
}

fn confirm_uninstall(daemon: &Daemon, source_path: &PathBuf) -> Result<bool> {
    Ok(dialoguer::Confirm::new()
        .with_prompt(format!("Uninstall service `{}` at `{}`?", daemon.name, source_path.display()))
        .interact()?)
}

fn bootout_service(daemon: &Daemon) {
    match launchctl::bootout(daemon) {
        Ok(result) => {
            if result.success() {
                stdout::success(&format!("Unloaded service: {}", daemon.name));
            } else if is_verbose() {
                stderr::warn(&format!("Warning: Failed to unload service: {}", result.stderr));
            }
        }
        Err(e) => {
            if is_verbose() {
                stderr::warn(&format!("Warning: Failed to unload service: {}", e));
            }
        }
    }
}

fn remove_plist_file(source_path: &PathBuf) -> Result<()> {
    let source_path_display = source_path.display().to_string();
    if source_path.exists() {
        std::fs::remove_file(source_path)?;
        stdout::success(&format!("Removed plist file: {}", source_path_display));
    } else if is_verbose() {
        stderr::warn(&format!("Warning: Plist file does not exist: {}", source_path_display));
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
    stdout::writeln(&format!("disabled_services: {:?}", disabled_services));

    let current_value = disabled_services.get(&daemon.name);
    if current_value.is_some() {
        if !confirm_db_overrides(daemon, current_value.unwrap())? {
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

fn confirm_db_overrides(daemon: &Daemon, current_value: &bool) -> Result<bool> {
    Ok(dialoguer::Confirm::new()
        .with_prompt(format!(
            "Found `{}` in enable/disable override database with disabled = {}. Remove it?",
            daemon.name, current_value
        ))
        .interact()?)
}
