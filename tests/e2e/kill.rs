use crate::harness::{self, TempServiceBuilder};

#[test]
fn kill_signals_running_service_and_keeps_it_loaded() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("kill").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running(&svc.target);

    harness::lctl()
        .args(["kill", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("sent TERM to {}", svc.label)));

    harness::assert_not_running(&svc.target);
    harness::assert_loaded(&svc.target);
}

#[test]
fn kill_refuses_service_without_a_process() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("kill-idle").sleep().build();
    svc.install_via_lctl();

    harness::lctl()
        .args(["kill", "-e", &svc.label])
        .assert()
        .failure()
        .stderr(predicates::str::contains(format!("{} is not running (idle)", svc.label)));
}

#[test]
fn kill_rejects_unknown_signal_with_hint() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("kill-badsig").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running(&svc.target);

    harness::lctl()
        .args(["kill", "-s", "NOPE", "-e", &svc.label])
        .assert()
        .failure()
        .stderr(predicates::str::contains("kill -l"));

    harness::assert_running(&svc.target);
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_kill_signals_running_service() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("kill").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running_sudo(&svc.target);

    harness::lctl_sudo()
        .args(["kill", "-s", "KILL", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("sent KILL to {}", svc.label)));

    harness::assert_not_running_sudo(&svc.target);
    harness::assert_loaded_sudo(&svc.target);
}
