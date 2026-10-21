use crate::harness::{self, TempServiceBuilder};

#[test]
fn enable_clears_disabled_entry_and_loads() {
    harness::require_macos();
    let mut svc = TempServiceBuilder::gui("enable").sleep().build();
    svc.install_via_lctl();

    harness::lctl().args(["disable", "-e", &svc.label]).assert().success();
    svc.mark_disabled();
    harness::assert_disabled(&svc.label);

    harness::lctl()
        .args(["enable", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("enabled {}", svc.label)));
    svc.disabled = false;

    harness::assert_enabled(&svc.label);
    harness::assert_loaded(&svc.target);

    harness::lctl().args(["start", "-e", &svc.label]).assert().success();
    harness::assert_running(&svc.target);
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_enable_clears_disabled_entry_and_loads() {
    harness::require_root_e2e();
    let mut svc = TempServiceBuilder::system("enable").sleep().build();
    svc.install_via_lctl();

    harness::lctl_sudo().args(["disable", "-e", &svc.label]).assert().success();
    svc.mark_disabled();
    harness::assert_disabled_system(&svc.label);

    harness::lctl_sudo()
        .args(["enable", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("enabled {}", svc.label)));
    svc.disabled = false;

    harness::assert_enabled_system(&svc.label);
    harness::assert_loaded_sudo(&svc.target);

    harness::lctl_sudo().args(["start", "-e", &svc.label]).assert().success();
    harness::assert_running_sudo(&svc.target);
}
