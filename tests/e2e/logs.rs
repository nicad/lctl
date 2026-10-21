use crate::harness::{self, TempServiceBuilder};

#[test]
fn logs_show_stdio_marker() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("logs").echo_then_sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running(&svc.target);

    // Wait briefly for the echo to flush to the log file (fail budget).
    let log = svc.stdout_log.as_ref().expect("stdout log path");
    assert!(
        harness::wait_until(
            || {
                std::fs::read_to_string(log).map(|s| s.contains("lctl-e2e-marker")).unwrap_or(false)
            },
            harness::FAIL_BUDGET
        ),
        "marker should appear in stdout log"
    );

    harness::lctl()
        .args(["logs", "-e", &svc.label, "-t", "1h"])
        .assert()
        .success()
        .stdout(predicates::str::contains("lctl-e2e-marker"));
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_logs_show_stdio_marker() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("logs").echo_then_sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running_sudo(&svc.target);

    let log = svc.stdout_log.as_ref().expect("stdout log path");
    assert!(
        harness::wait_until(
            || {
                std::fs::read_to_string(log).map(|s| s.contains("lctl-e2e-marker")).unwrap_or(false)
            },
            harness::FAIL_BUDGET
        ),
        "marker should appear in stdout log"
    );

    harness::lctl_sudo()
        .args(["logs", "-e", &svc.label, "-t", "1h"])
        .assert()
        .success()
        .stdout(predicates::str::contains("lctl-e2e-marker"));
}
