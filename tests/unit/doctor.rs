use lctl::doctor::{Check, Severity, check, check_state};
use lctl::parser::parse_print_service;
use lctl::types::{Daemon, Domain, ForWhom, Plist};
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

const LABEL: &str = "com.example.doctor";

fn plist(program: &[&str]) -> Plist {
    Plist {
        label: Some(LABEL.into()),
        program_arguments: Some(program.iter().map(|s| s.to_string()).collect()),
        run_at_load: Some(true),
        ..Default::default()
    }
}

/// A daemon whose plist file exists in a temp dir, owned by the current user, mode 644.
fn daemon_on_disk(plist: Plist, dir: &TempDir) -> Daemon {
    let path = dir.path().join(format!("{LABEL}.plist"));
    std::fs::write(&path, "<plist/>").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    let uid = nix::unistd::getuid().as_raw();
    Daemon::from_plist(plist, Some(path), Domain::Gui(uid), ForWhom::User, uid)
}

fn problems(checks: &[Check]) -> Vec<&Check> {
    checks.iter().filter(|check| check.severity != Severity::Ok).collect()
}

fn find<'a>(checks: &'a [Check], text: &str) -> &'a Check {
    checks
        .iter()
        .find(|check| check.message.contains(text))
        .unwrap_or_else(|| panic!("no check containing {text:?} in {checks:?}"))
}

#[test]
fn healthy_service_has_no_problems() {
    let dir = TempDir::new().unwrap();
    let mut daemon = daemon_on_disk(plist(&["/bin/sleep", "1"]), &dir);
    daemon.loaded = true;
    let checks = check(&daemon, None);
    assert!(problems(&checks).is_empty(), "{checks:?}");
}

#[test]
fn missing_program_fails() {
    let dir = TempDir::new().unwrap();
    let daemon = daemon_on_disk(plist(&["/nonexistent/lctl-doctor-test"]), &dir);
    let checks = check(&daemon, None);
    assert_eq!(find(&checks, "program not found").severity, Severity::Fail);
}

#[test]
fn bare_program_is_looked_up_in_launchd_path() {
    let dir = TempDir::new().unwrap();
    let found = check(&daemon_on_disk(plist(&["sleep", "1"]), &dir), None);
    assert_eq!(find(&found, "found in PATH").severity, Severity::Ok);
    let missing = check(&daemon_on_disk(plist(&["lctl-no-such-tool"]), &dir), None);
    assert_eq!(find(&missing, "not found in launchd's PATH").severity, Severity::Fail);
}

#[test]
fn no_trigger_warns_with_start_fix() {
    let dir = TempDir::new().unwrap();
    let daemon = daemon_on_disk(Plist { run_at_load: None, ..plist(&["/bin/sleep"]) }, &dir);
    let checks = check(&daemon, None);
    let trigger = find(&checks, "no launch trigger");
    assert_eq!(trigger.severity, Severity::Warn);
    assert_eq!(trigger.fix.as_deref(), Some("lctl start com.example.doctor"));
}

#[test]
fn missing_log_directory_warns_with_mkdir_fix() {
    let dir = TempDir::new().unwrap();
    let log = dir.path().join("missing/out.log").to_string_lossy().to_string();
    let daemon =
        daemon_on_disk(Plist { standard_out_path: Some(log), ..plist(&["/bin/sleep"]) }, &dir);
    let checks = check(&daemon, None);
    let missing = find(&checks, "log directory doesn't exist");
    assert_eq!(missing.severity, Severity::Warn);
    assert!(missing.fix.as_deref().unwrap().starts_with("mkdir -p "), "{missing:?}");
}

#[test]
fn plist_without_label_only_reports_that() {
    let dir = TempDir::new().unwrap();
    let daemon = daemon_on_disk(Plist::default(), &dir);
    let checks = check(&daemon, None);
    assert_eq!(checks.len(), 1, "{checks:?}");
    assert_eq!(find(&checks, "no Label").severity, Severity::Fail);
}

#[test]
fn group_writable_plist_fails() {
    let dir = TempDir::new().unwrap();
    let daemon = daemon_on_disk(plist(&["/bin/sleep"]), &dir);
    let path = daemon.source_path.clone().unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o664)).unwrap();
    let checks = check(&daemon, None);
    assert_eq!(find(&checks, "not writable by group/others").severity, Severity::Fail);
}

#[test]
fn short_start_interval_warns() {
    let dir = TempDir::new().unwrap();
    let daemon = daemon_on_disk(Plist { start_interval: Some(5), ..plist(&["/bin/sleep"]) }, &dir);
    assert_eq!(find(&check(&daemon, None), "StartInterval of 5s").severity, Severity::Warn);
}

fn print_with_arguments(arguments: &[&str]) -> String {
    let mut print =
        format!("gui/501/{LABEL} = {{\n\tprogram = {}\n\targuments = {{\n", arguments[0]);
    for argument in arguments {
        print.push_str(&format!("\t\t{argument}\n"));
    }
    print.push_str("\t}\n}\n");
    print
}

#[test]
fn plist_edited_since_load_warns_with_reload_fix() {
    let dir = TempDir::new().unwrap();
    let mut daemon = daemon_on_disk(plist(&["/bin/sleep", "2"]), &dir);
    daemon.loaded = true;

    let stale = print_with_arguments(&["/bin/sleep", "1"]);
    let checks = check(&daemon, Some(&parse_print_service(&stale).unwrap()));
    let changed = find(&checks, "changed on disk");
    assert_eq!(changed.fix.as_deref(), Some("lctl reload com.example.doctor"));

    let current = print_with_arguments(&["/bin/sleep", "2"]);
    let checks = check(&daemon, Some(&parse_print_service(&current).unwrap()));
    assert!(problems(&checks).is_empty(), "{checks:?}");
}

#[test]
fn state_checks_point_to_the_fix() {
    let dir = TempDir::new().unwrap();
    let mut daemon = daemon_on_disk(plist(&["/bin/sleep"]), &dir);

    daemon.disabled = true;
    assert_eq!(check_state(&daemon).fix.as_deref(), Some("lctl enable com.example.doctor"));

    daemon.disabled = false;
    daemon.loaded = true;
    daemon.last_exit_code = Some("78".into());
    let failed = check_state(&daemon);
    assert_eq!(failed.severity, Severity::Warn);
    assert_eq!(failed.fix.as_deref(), Some("lctl logs com.example.doctor"));

    daemon.pid = Some(123);
    assert_eq!(check_state(&daemon).severity, Severity::Ok);
}
