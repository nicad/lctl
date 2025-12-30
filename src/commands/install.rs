use crate::angel::Angel;
use crate::cli::InstallArgs;
use crate::error::{AngelError, Result, UserError};
use crate::launchctl;
use crate::output::stdout;
use crate::types::{Daemon, Domain, ForWhom, Plist};
use clap::ValueEnum;
use nix::fcntl::{AT_FDCWD, AtFlags};
use nix::sys::stat::{FchmodatFlags, Mode, fchmodat};
use nix::unistd::{Gid, Uid, fchownat};
use std::path::{Path, PathBuf};

#[derive(Clone, ValueEnum, PartialEq)]
pub enum InstallStrategy {
    Copy,
    Symlink,
    Move,
}

pub fn run(angel: &Angel, args: &InstallArgs) -> Result<()> {
    let source_path = PathBuf::from(args.path.as_str());
    stdout::writeln(&format!("source_path: {}", source_path.display()));
    let bytes = std::fs::read(&source_path)?;

    let plist_data = plist::from_bytes::<Plist>(&bytes)?;
    let service_name = plist_data
        .label
        .clone()
        .ok_or_else(|| UserError::InvalidArgument("No label found in plist".to_string()))?;

    // ask user which domain
    let selected_domain = get_domain_selection(angel, &plist_data)?;

    // copy/symlink/move
    let target_path = make_target_path(&selected_domain, &service_name)?;

    install_file(&args.strategy, &source_path, &target_path, &selected_domain)?;

    // kill running service if it is running
    kill_running_service(angel, &service_name)?;

    let daemon = Daemon::from_plist(
        plist_data,
        Some(target_path.clone()),
        selected_domain,
        ForWhom::User,
        angel.uid.as_raw(),
    );

    let result = launchctl::bootstrap(&daemon)?;
    match result.success() {
        true => stdout::success(&format!("installed {}", daemon.name)),
        false => stdout::error(&format!("failed to install {}: {}", daemon.name, result.stderr)),
    }
    Ok(())
}

fn kill_running_service(angel: &Angel, service_name: &str) -> Result<()> {
    let daemon = match angel.daemons.get_match(service_name, true) {
        Ok(daemon) => match daemon.pid {
            Some(_) => daemon,
            None => return Ok(()), // not running. proceed.
        },
        Err(AngelError::User(UserError::DaemonNotFound(_))) => return Ok(()), // not found. proceed.
        Err(e) => return Err(e.into()),
    };
    confirm_kill_running_service(&daemon)?;
    launchctl::disable(daemon)?; // disable before bootout to prevent restart when keepAlive = true
    launchctl::bootout(daemon)?;
    launchctl::enable(daemon)?;
    Ok(())
}

fn confirm_kill_running_service(daemon: &Daemon) -> Result<()> {
    dialoguer::Confirm::new()
        .with_prompt(format!(
            "A service with the name {} is already running. Run bootout?",
            daemon.name
        ))
        .interact()
        .unwrap_or(false)
        .then_some(())
        .ok_or_else(|| {
            UserError::InvalidArgument(format!(
                "A service with the name {} is already running. Bootstrap will fail.",
                daemon.name
            ))
            .into()
        })
}

fn install_file(
    strategy: &InstallStrategy,
    source_path: &Path,
    target_path: &Path,
    domain: &Domain,
) -> Result<()> {
    confirm_overwrite(target_path)?;

    if *strategy != InstallStrategy::Copy && *domain == Domain::System {
        set_system_permissions(source_path)?;
    }

    match strategy {
        InstallStrategy::Symlink => std::os::unix::fs::symlink(source_path, target_path)?,
        InstallStrategy::Move => std::fs::rename(source_path, target_path)?,
        InstallStrategy::Copy => std::fs::copy(source_path, target_path).map(|_| ())?,
    }

    if *strategy != InstallStrategy::Move && *domain == Domain::System {
        set_system_permissions(target_path)?;
    }

    Ok(())
}

fn confirm_overwrite(target_path: &Path) -> Result<()> {
    if !target_path.exists() {
        return Ok(());
    }
    let choice = dialoguer::Confirm::new()
        .with_prompt(format!("A file already exists at {}. Overwrite it?", target_path.display()))
        .default(true)
        .interact()
        .unwrap_or(false);

    if !choice {
        return Err(UserError::InvalidArgument(format!(
            "A file already exists at {}",
            target_path.display()
        ))
        .into());
    }
    std::fs::remove_file(target_path)?;
    Ok(())
}

fn get_domain_selection(angel: &Angel, plist_data: &Plist) -> Result<Domain> {
    let domains =
        [Domain::Gui(angel.uid.as_raw()), Domain::System, Domain::User(angel.uid.as_raw())];
    let domain_selection_index = dialoguer::Select::new()
        .with_prompt("In which domain should the service be installed?")
        .items(&domains)
        .default(0)
        .interact()?;

    let selected_domain = domains[domain_selection_index].clone();
    let plist_domain = Domain::from_plist(plist_data, angel.uid.as_raw(), selected_domain.clone());
    match plist_domain == selected_domain {
        false => {
            return Err(UserError::InvalidArgument(format!(
                "The domain written in the plist is `{}` does not match the selected domain `{}`.",
                plist_domain, selected_domain
            ))
            .into());
        }
        true => {}
    }

    match selected_domain {
        Domain::System => match angel.is_root() {
            false => return Err(UserError::RequiresRoot.into()),
            true => {}
        },
        _ => {}
    }

    Ok(selected_domain)
}

fn make_target_path(domain: &Domain, service_name: &str) -> Result<PathBuf> {
    let target_dir = match domain {
        Domain::System => PathBuf::from("/Library/LaunchDaemons"),
        _ => crate::utils::get_home().map(|home| home.join("Library/LaunchAgents")).ok_or_else(
            || UserError::InvalidArgument("Could not determine user home directory".to_string()),
        )?,
    };
    let filename = match service_name.ends_with(".plist") {
        true => service_name.to_string(),
        false => format!("{}.plist", service_name),
    };
    Ok(target_dir.join(filename))
}

fn set_system_permissions(path: &Path) -> Result<()> {
    // sudo chown root:wheel
    let uid = Uid::from(0); // root = 0
    let gid = Gid::from_raw(0); // wheel = 0
    fchownat(AT_FDCWD, path, Some(uid), Some(gid), AtFlags::AT_SYMLINK_NOFOLLOW)?;

    // sudo chmod 644
    let mode = Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH; // 644
    fchmodat(AT_FDCWD, path, mode, FchmodatFlags::FollowSymlink)?;
    Ok(())
}
