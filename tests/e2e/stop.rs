use crate::harness::{self, TempServiceBuilder};

#[test]
fn stop_unloads_running_service() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("stop").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running(&svc.target);

    harness::lctl()
        .args(["stop", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("stopped {}", svc.label)));

    harness::assert_not_loaded(&svc.target);
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_stop_unloads_running_service() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("stop").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running_sudo(&svc.target);

    harness::lctl_sudo()
        .args(["stop", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("stopped {}", svc.label)));

    harness::assert_not_loaded_sudo(&svc.target);
}
