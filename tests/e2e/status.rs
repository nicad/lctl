use crate::fixtures::ProgramKind;
use crate::harness::{self, TempServiceBuilder};

#[test]
fn status_shows_running_state() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("status").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running(&svc.target);

    harness::lctl()
        .args(["status", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(&svc.label))
        .stdout(predicates::str::contains("running"))
        .stdout(predicates::str::contains("/bin/sleep 999999"))
        .stdout(predicates::str::contains("(up "))
        .stdout(predicates::str::contains("lctl stop"));
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
fn status_flags_plist_changed_since_load() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("status-changed").build();
    svc.install_via_lctl();
    svc.rewrite_fixture(ProgramKind::Sleep, false);

    harness::lctl()
        .args(["status", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains("changed on disk"))
        .stdout(predicates::str::contains(format!("lctl reload {}", svc.label)));
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
        .stdout(predicates::str::contains("running"))
        .stdout(predicates::str::contains("/bin/sleep 999999"));
}
