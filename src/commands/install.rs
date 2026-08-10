use crate::angel::Angel;
use crate::cli::{InstallArgs, InstallStrategy};
use crate::error::{Error, Result, err_msg};
use crate::launchctl;
use crate::output::{is_verbose, stderr, stdout};
use crate::styles;
use crate::types::{Daemon, Domain, ForWhom, Plist};
use crate::utils;
use nix::fcntl::{AT_FDCWD, AtFlags};
use nix::sys::stat::{FchmodatFlags, Mode, fchmodat};
use nix::unistd::{Gid, Uid, fchownat};
use std::path::{Path, PathBuf};

pub(crate) fn run(angel: &Angel, args: &InstallArgs) -> Result<()> {
    let expanded = utils::expand_path(args.path.as_str())?;
    let (source_path, preselected_domain) =
        resolve_source_path(angel, &expanded, args.path.as_str())?;
    let bytes = std::fs::read(&source_path)?;

    let plist_data = plist::from_bytes::<Plist>(&bytes)?;
    let service_name = plist_data
        .label
        .clone()
        .ok_or_else(|| Error::InvalidInput(err_msg!("No label found in plist")))?;

    stdout::hint(&format!(
        "Installing {} from {}",
        service_name,
        styles::display_source_path(&source_path, is_verbose())
    ));

    let selected_domain = get_domain_selection(angel, &plist_data, preselected_domain)?;

    // copy/symlink/move
    let target_path = make_target_path(angel, &selected_domain, &service_name)?;
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
    result.require_ok(&format!("install {}", daemon.name))?;
    stdout::success(&format!("installed {}", daemon.name));
    Ok(())
}

/// Local plist path: existing file or symlink (including dangling). Otherwise try fuzzy daemon name
/// match (covers plists already loaded from `~/.config/angel/{user,system,gui}`). Symlinks are
/// followed; `preselected_domain` skips the domain prompt when path is under those angel dirs.
fn resolve_source_path(
    angel: &Angel,
    expanded: &Path,
    query: &str,
) -> Result<(PathBuf, Option<Domain>)> {
    let is_local_plist = expanded.is_file()
        || std::fs::symlink_metadata(expanded).map(|m| m.file_type().is_symlink()).unwrap_or(false);

    let path = if is_local_plist {
        expanded.to_path_buf()
    } else {
        let query = query.strip_suffix(".plist").unwrap_or(query);
        let daemon = super::resolve_angel_daemon(angel, query)?;
        daemon.require_source_path()?.to_path_buf()
    };

    if path.is_symlink() {
        stderr::warn("source is a symlink; this may cause problems");
    }

    let resolved = std::fs::canonicalize(&path).map_err(|e| {
        Error::InvalidInput(err_msg!("Could not resolve source path `{}`: {}", path.display(), e))
    })?;
    let preselected_domain = infer_domain_from_angel_path(angel, &resolved);
    Ok((resolved, preselected_domain))
}

fn infer_domain_from_angel_path(angel: &Angel, resolved: &Path) -> Option<Domain> {
    for (base, domain, _) in crate::angel::angel_domain_dirs(&angel.angel_dir, angel.uid.as_raw()) {
        if let Ok(base_canon) = std::fs::canonicalize(&base) {
            if resolved.starts_with(&base_canon) {
                return Some(domain);
            }
        } else if resolved.starts_with(&base) {
            return Some(domain);
        }
    }
    None
}

fn kill_running_service(angel: &Angel, service_name: &str) -> Result<()> {
    let daemon = match super::resolve_daemon(angel, service_name, true) {
        Ok(daemon) => match daemon.pid {
            Some(_) => daemon,
            None => return Ok(()), // not running. proceed.
        },
        Err(Error::NotFound(_)) => return Ok(()), // not found. proceed.
        Err(e) => return Err(e),
    };
    confirm_kill_running_service(&daemon)?;
    launchctl::disable(daemon)?; // disable before bootout to prevent restart when keepAlive = true
    launchctl::bootout(daemon)?;
    launchctl::enable(daemon)?;
    Ok(())
}

fn confirm_kill_running_service(daemon: &Daemon) -> Result<()> {
    if utils::confirm(&format!(
        "A service with the name {} is already running. Run bootout?",
        daemon.name
    ))? {
        Ok(())
    } else {
        Err(Error::InvalidInput(err_msg!(
            "A service with the name {} is already running. Bootstrap will fail.",
            daemon.name
        )))
    }
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
    // Use symlink_metadata, not exists(): exists() is false for dangling symlinks,
    // but symlink() still fails with EEXIST if that path is occupied.
    if target_path.symlink_metadata().is_err() {
        return Ok(());
    }
    if !utils::confirm_with_default(
        &format!("A file already exists at {}.\nOverwrite it?", target_path.display()),
        true,
    )? {
        return Err(Error::InvalidInput(err_msg!(
            "A file already exists at {}",
            styles::display_source_path(target_path, is_verbose())
        )));
    }
    std::fs::remove_file(target_path)?;
    Ok(())
}

fn get_domain_selection(
    angel: &Angel,
    plist_data: &Plist,
    preselected: Option<Domain>,
) -> Result<Domain> {
    let domains =
        [Domain::Gui(angel.uid.as_raw()), Domain::System, Domain::User(angel.uid.as_raw())];
    let selected_domain = match preselected {
        Some(d) => d,
        None => {
            let domain_selection_index = dialoguer::Select::new()
                .with_prompt("In which domain should the service be installed?")
                .items(&domains)
                .default(0)
                .interact()?;
            domains[domain_selection_index]
        }
    };
    let plist_domain = Domain::from_plist(plist_data, angel.uid.as_raw(), selected_domain);
    if plist_domain != selected_domain {
        return Err(Error::InvalidInput(err_msg!(
            "The domain written in the plist is `{}` does not match the selected domain `{}`.",
            plist_domain,
            selected_domain
        )));
    }

    if matches!(selected_domain, Domain::System) && !angel.is_root() {
        return Err(Error::NeedsRoot);
    }

    Ok(selected_domain)
}

fn make_target_path(angel: &Angel, domain: &Domain, service_name: &str) -> Result<PathBuf> {
    let target_dir = match domain {
        Domain::System => PathBuf::from("/Library/LaunchDaemons"),
        _ => angel.home.join("Library/LaunchAgents"),
    };
    let filename = if service_name.ends_with(".plist") {
        service_name.to_string()
    } else {
        format!("{}.plist", service_name)
    };
    Ok(target_dir.join(filename))
}

fn set_system_permissions(path: &Path) -> Result<()> {
    // sudo chown root:wheel
    let uid = Uid::from_raw(0); // root = 0
    let gid = Gid::from_raw(0); // wheel = 0
    fchownat(AT_FDCWD, path, Some(uid), Some(gid), AtFlags::AT_SYMLINK_NOFOLLOW)?;

    // sudo chmod 644
    let mode = Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH; // 644
    fchmodat(AT_FDCWD, path, mode, FchmodatFlags::FollowSymlink)?;
    Ok(())
}
