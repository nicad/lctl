use crate::cli::NameArgs;
use crate::error::{Error, Result, err_msg};
use crate::launchctl;
use crate::lctl::Lctl;
use crate::output::{is_verbose, stderr, stdout};
use crate::parser::{self, ServiceItem};
use crate::types::Daemon;
use crate::utils;
use std::path::{Path, PathBuf};

pub(crate) fn run(lctl: &Lctl, args: &NameArgs) -> Result<()> {
    let daemon = super::resolve_daemon(lctl, &args.name, args.exact)?;

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
    clear_disabled_override(daemon);

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
        _ => Err(Error::InvalidInput(err_msg!("Service does not have an installed plist file"))),
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
    // Use symlink_metadata, not exists(): exists() is false for dangling symlinks.
    if source_path.symlink_metadata().is_ok() {
        std::fs::remove_file(source_path)?;
        stdout::success(&format!("Removed plist file: {}", source_path_display));
    } else if is_verbose() {
        stderr::warn(&format!("Plist file does not exist: {}", source_path_display));
    }
    Ok(())
}

/// Best-effort: clear disable overrides via launchctl enable. Never fails uninstall.
fn clear_disabled_override(daemon: &Daemon) {
    match launchctl::enable(daemon) {
        Ok(result) if result.success() => {
            if is_verbose() {
                stdout::success(&format!("Cleared disable override for {}", daemon.name));
            }
        }
        Ok(result) => {
            let detail = if result.stderr.is_empty() {
                format!("exit code {:?}", result.exit_code)
            } else {
                result.stderr
            };
            stderr::warn(&format!(
                "Could not clear disable override for {}: {}",
                daemon.name, detail
            ));
        }
        Err(e) => {
            stderr::warn(&format!("Could not clear disable override for {}: {}", daemon.name, e));
        }
    }
}
