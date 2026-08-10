use crate::harness::{self, TempServiceBuilder};

#[test]
fn stop_kills_running_service() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("stop").sleep().run_at_load().build();
    svc.install_via_angel();
    harness::assert_running(&svc.target);

    harness::angel()
        .args(["stop", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("stopped {}", svc.label)));

    harness::assert_not_running(&svc.target);
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_stop_kills_running_service() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("stop").sleep().run_at_load().build();
    svc.install_via_angel();
    harness::assert_running_sudo(&svc.target);

    harness::angel_sudo()
        .args(["stop", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("stopped {}", svc.label)));

    harness::assert_not_running_sudo(&svc.target);
}
