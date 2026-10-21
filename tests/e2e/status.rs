use crate::harness::{self, TempServiceBuilder};
use predicates::prelude::*;

#[test]
fn status_shows_running_state() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("status").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running(&svc.target);

    // Match table "State … running" (not "not running"). Dot line has ANSI between ● and text.
    harness::lctl()
        .args(["status", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(&svc.label))
        .stdout(predicates::str::is_match(r"(?m)State\s+running\b").unwrap())
        .stdout(predicates::str::contains("not running").not());
}

#[test]
fn status_reports_not_loaded() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("status-unloaded").build();
    svc.place_gui_plist_unloaded();

    harness::lctl()
        .args(["status", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(&svc.label))
        .stdout(predicates::str::contains("not loaded"));
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_status_shows_running_state() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("status").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running_sudo(&svc.target);

    harness::lctl_sudo()
        .args(["status", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(&svc.label))
        .stdout(predicates::str::is_match(r"(?m)State\s+running\b").unwrap())
        .stdout(predicates::str::contains("not running").not());
}
