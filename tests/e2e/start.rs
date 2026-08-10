use crate::harness::{self, TempServiceBuilder};

#[test]
fn start_kickstarts_sleep_service() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("start").sleep().build();
    svc.install_via_angel();
    harness::assert_not_running(&svc.target);

    harness::angel()
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
    svc.install_via_angel();
    harness::assert_not_running_sudo(&svc.target);

    harness::angel_sudo()
        .args(["start", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("started {}", svc.label)));

    harness::assert_running_sudo(&svc.target);
}
