use crate::cli::{InstallArgs, InstallDomain, InstallStrategy};
use crate::doctor;
use crate::error::{Error, Result, err_msg};
use crate::launchctl;
use crate::lctl::Lctl;
use crate::output::{is_verbose, stderr, stdout};
use crate::styles;
use crate::sudo;
use crate::types::{Daemon, Domain, ForWhom, Plist};
use crate::utils;
use nix::fcntl::{AT_FDCWD, AtFlags};
use nix::sys::stat::{FchmodatFlags, Mode, fchmodat};
use nix::unistd::{Gid, Uid, fchownat};
use std::path::{Path, PathBuf};

pub(crate) fn run(lctl: &Lctl, args: &InstallArgs) -> Result<()> {
    let expanded = utils::expand_path(args.path.as_str())?;
    let (source_path, preselected_domain) =
        resolve_source_path(lctl, &expanded, args.path.as_str())?;
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

    let selected_domain = get_domain_selection(lctl, &plist_data, args.domain, preselected_domain)?;

    // copy/symlink/move
    let target_path = make_target_path(lctl, &selected_domain, &service_name)?;
    install_file(&args.strategy, &source_path, &target_path, &selected_domain)?;

    free_label(lctl, &service_name)?;

    let daemon = Daemon::from_plist(
        plist_data,
        Some(target_path.clone()),
        selected_domain,
        ForWhom::User,
        lctl.uid.as_raw(),
    );

    super::step(&daemon, launchctl::bootstrap, "install")?;
    stdout::success(&format!("installed {}", daemon.name));

    let installed = Daemon { loaded: true, ..daemon };
    for check in doctor::check(&installed, None).iter().filter(|check| check.is_problem()) {
        stdout::writeln(&format!("  {}", styles::check(check)));
    }
    Ok(())
}

/// Local plist path: existing file or symlink (including dangling). Otherwise try fuzzy daemon name
/// match (covers plists already loaded from `{lctl_dir}/{user,system,gui}`). Symlinks are
/// followed; `preselected_domain` skips the domain prompt when path is under those lctl dirs.
fn resolve_source_path(
    lctl: &Lctl,
    expanded: &Path,
    query: &str,
) -> Result<(PathBuf, Option<Domain>)> {
    let is_local_plist = expanded.is_file()
        || std::fs::symlink_metadata(expanded).map(|m| m.file_type().is_symlink()).unwrap_or(false);

    let path = if is_local_plist {
        expanded.to_path_buf()
    } else {
        let query = query.strip_suffix(".plist").unwrap_or(query);
        let daemon = super::resolve_lctl_daemon(lctl, query)?;
        daemon.require_source_path()?.to_path_buf()
    };

    if path.is_symlink() {
        stderr::warn("source is a symlink; this may cause problems");
    }

    let resolved = std::fs::canonicalize(&path).map_err(|e| {
        Error::InvalidInput(err_msg!("Could not resolve source path `{}`: {}", path.display(), e))
    })?;
    let preselected_domain = infer_domain_from_lctl_path(lctl, &resolved);
    Ok((resolved, preselected_domain))
}

fn infer_domain_from_lctl_path(lctl: &Lctl, resolved: &Path) -> Option<Domain> {
    for (base, domain, _) in crate::lctl::lctl_domain_dirs(&lctl.lctl_dir, lctl.uid.as_raw()) {
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

/// Make the bootstrap possible: unload a loaded service with the same label (after asking),
/// and clear a leftover disabled flag.
fn free_label(lctl: &Lctl, service_name: &str) -> Result<()> {
    let daemon = match super::resolve_daemon(lctl, service_name, true) {
        Ok(daemon) => daemon,
        Err(Error::NotFound(_)) => return Ok(()),
        Err(e) => return Err(e),
    };
    if daemon.loaded {
        confirm_unload_existing_service(daemon)?;
        super::disable_and_unload(daemon)?;
    } else if daemon.disabled {
        stdout::hint(&format!("{} was disabled; enabling it", daemon.name));
    } else {
        return Ok(());
    }
    super::step(daemon, launchctl::enable, "enable")
}

fn confirm_unload_existing_service(daemon: &Daemon) -> Result<()> {
    if utils::confirm(&format!(
        "A service with the name {} is already loaded. Unload it?",
        daemon.name
    ))? {
        Ok(())
    } else {
        Err(Error::InvalidInput(err_msg!(
            "A service with the name {} is already loaded. Bootstrap will fail.",
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
    let system = matches!(domain, Domain::System);
    confirm_overwrite(target_path, system)?;

    if *strategy != InstallStrategy::Copy && system {
        set_system_permissions(source_path)?;
    }

    place_file(strategy, source_path, target_path, system)?;

    if *strategy != InstallStrategy::Move && system {
        set_system_permissions(target_path)?;
    }

    Ok(())
}

fn place_file(
    strategy: &InstallStrategy,
    source_path: &Path,
    target_path: &Path,
    system: bool,
) -> Result<()> {
    if system && !sudo::is_root() {
        let src = path_str(source_path)?;
        let dest = path_str(target_path)?;
        match strategy {
            InstallStrategy::Symlink => {
                sudo::require_sudo_n(&["/bin/ln", "-sf", src, dest], "symlink install")?
            }
            InstallStrategy::Move => sudo::require_sudo_n(&["/bin/mv", src, dest], "move install")?,
            InstallStrategy::Copy => sudo::require_sudo_n(&["/bin/cp", src, dest], "copy install")?,
        }
        return Ok(());
    }

    match strategy {
        InstallStrategy::Symlink => std::os::unix::fs::symlink(source_path, target_path)?,
        InstallStrategy::Move => std::fs::rename(source_path, target_path)?,
        InstallStrategy::Copy => std::fs::copy(source_path, target_path).map(|_| ())?,
    }
    Ok(())
}

fn confirm_overwrite(target_path: &Path, system: bool) -> Result<()> {
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
    remove_path(target_path, system)
}

fn remove_path(path: &Path, system: bool) -> Result<()> {
    if system && !sudo::is_root() {
        sudo::require_sudo_n(&["/bin/rm", "-f", path_str(path)?], "remove existing install")?;
        return Ok(());
    }
    std::fs::remove_file(path)?;
    Ok(())
}

fn get_domain_selection(
    lctl: &Lctl,
    plist_data: &Plist,
    cli_domain: Option<InstallDomain>,
    preselected: Option<Domain>,
) -> Result<Domain> {
    let uid = lctl.uid.as_raw();
    let domains = [Domain::Gui(uid), Domain::System, Domain::User(uid)];
    let selected_domain = match (cli_domain, preselected) {
        (Some(InstallDomain::Gui), _) => Domain::Gui(uid),
        (Some(InstallDomain::User), _) => Domain::User(uid),
        (Some(InstallDomain::System), _) => Domain::System,
        (None, Some(d)) => d,
        (None, None) => {
            let domain_selection_index = dialoguer::Select::new()
                .with_prompt("In which domain should the service be installed?")
                .items(&domains)
                .default(0)
                .interact()?;
            domains[domain_selection_index]
        }
    };
    let plist_domain = Domain::from_plist(plist_data, lctl.uid.as_raw(), selected_domain);
    if plist_domain != selected_domain {
        return Err(Error::InvalidInput(err_msg!(
            "The domain written in the plist is `{}` does not match the selected domain `{}`.",
            plist_domain,
            selected_domain
        )));
    }

    if matches!(selected_domain, Domain::System) {
        // Prompt on TTY when needed; non-TTY without root → NeedsRoot.
        sudo::ensure_elevated()?;
    }

    Ok(selected_domain)
}

fn make_target_path(lctl: &Lctl, domain: &Domain, service_name: &str) -> Result<PathBuf> {
    let target_dir = match domain {
        Domain::System => PathBuf::from("/Library/LaunchDaemons"),
        _ => lctl.home.join("Library/LaunchAgents"),
    };
    let filename = if service_name.ends_with(".plist") {
        service_name.to_string()
    } else {
        format!("{}.plist", service_name)
    };
    Ok(target_dir.join(filename))
}

fn set_system_permissions(path: &Path) -> Result<()> {
    if sudo::is_root() {
        let uid = Uid::from_raw(0); // root = 0
        let gid = Gid::from_raw(0); // wheel = 0
        fchownat(AT_FDCWD, path, Some(uid), Some(gid), AtFlags::AT_SYMLINK_NOFOLLOW)?;

        let mode = Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH; // 644
        fchmodat(AT_FDCWD, path, mode, FchmodatFlags::FollowSymlink)?;
        return Ok(());
    }

    let path_str = path_str(path)?;
    sudo::require_sudo_n(&["/usr/sbin/chown", "-h", "root:wheel", path_str], "chown root:wheel")?;
    sudo::require_sudo_n(&["/bin/chmod", "644", path_str], "chmod 644")?;
    Ok(())
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str().ok_or_else(|| Error::InvalidInput(err_msg!("Invalid path encoding")))
}
