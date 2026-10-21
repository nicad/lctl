use crate::harness::{self, TempServiceBuilder};

#[test]
fn start_kickstarts_sleep_service() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("start").sleep().build();
    svc.install_via_lctl();
    harness::assert_not_running(&svc.target);

    harness::lctl()
        .args(["start", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("started {}", svc.label)));

    harness::assert_running(&svc.target);
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_start_kickstarts_sleep_service() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("start").sleep().build();
    svc.install_via_lctl();
    harness::assert_not_running_sudo(&svc.target);

    harness::lctl_sudo()
        .args(["start", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("started {}", svc.label)));

    harness::assert_running_sudo(&svc.target);
}

#[test]
fn start_loads_unloaded_plist() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("start-unloaded").sleep().build();
    svc.place_gui_plist_unloaded();
    harness::assert_not_loaded(&svc.target);

    harness::lctl()
        .args(["start", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("started {}", svc.label)));

    harness::assert_running(&svc.target);
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_start_loads_unloaded_plist() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("start-unloaded").sleep().build();
    svc.place_system_plist_unloaded();
    harness::assert_not_loaded_sudo(&svc.target);

    harness::lctl_sudo()
        .args(["start", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("started {}", svc.label)));

    harness::assert_running_sudo(&svc.target);
}

#[test]
fn start_on_disabled_service_says_enable_first() {
    harness::require_macos();
    let mut svc = TempServiceBuilder::gui("start-disabled").sleep().build();
    svc.install_via_lctl();
    harness::lctl().args(["disable", "-e", &svc.label]).assert().success();
    svc.mark_disabled();

    harness::lctl()
        .args(["start", "-e", &svc.label])
        .assert()
        .failure()
        .stderr(predicates::str::contains(format!("lctl enable {}", svc.label)));
}
