use crate::harness::{self, TempServiceBuilder};

#[test]
fn enable_clears_disabled_domain_entry() {
    harness::require_macos();
    let mut svc = TempServiceBuilder::gui("enable").sleep().build();
    svc.install_via_angel();

    harness::angel()
        .args(["disable", "-e", &svc.label])
        .assert()
        .success();
    svc.mark_disabled();
    harness::assert_disabled(&svc.label);

    harness::angel()
        .args(["enable", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("enabled {}", svc.label)));
    svc.disabled = false;

    harness::assert_enabled(&svc.label);

    harness::angel()
        .args(["start", "-e", &svc.label])
        .assert()
        .success();
    harness::assert_running(&svc.target);
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_enable_clears_disabled_domain_entry() {
    harness::require_root_e2e();
    let mut svc = TempServiceBuilder::system("enable").sleep().build();
    svc.install_via_angel();

    harness::angel_sudo()
        .args(["disable", "-e", &svc.label])
        .assert()
        .success();
    svc.mark_disabled();
    harness::assert_disabled_system(&svc.label);

    harness::angel_sudo()
        .args(["enable", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("enabled {}", svc.label)));
    svc.disabled = false;

    harness::assert_enabled_system(&svc.label);

    harness::angel_sudo()
        .args(["start", "-e", &svc.label])
        .assert()
        .success();
    harness::assert_running_sudo(&svc.target);
}
