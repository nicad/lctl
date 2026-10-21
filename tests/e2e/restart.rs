use crate::harness::{self, TempServiceBuilder};

#[test]
fn restart_replaces_pid() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("restart").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running(&svc.target);
    let before = harness::print_service(&svc.target)
        .and_then(|s| harness::pid_from_print(&s))
        .expect("pid before restart");

    harness::lctl()
        .args(["restart", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("restarted {}", svc.label)));

    harness::assert_running(&svc.target);
    let after = harness::print_service(&svc.target)
        .and_then(|s| harness::pid_from_print(&s))
        .expect("pid after restart");
    assert_ne!(before, after, "restart should replace the process");
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_restart_replaces_pid() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("restart").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running_sudo(&svc.target);
    let before = harness::print_service_sudo(&svc.target)
        .and_then(|s| harness::pid_from_print(&s))
        .expect("pid before restart");

    harness::lctl_sudo()
        .args(["restart", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("restarted {}", svc.label)));

    harness::assert_running_sudo(&svc.target);
    let after = harness::print_service_sudo(&svc.target)
        .and_then(|s| harness::pid_from_print(&s))
        .expect("pid after restart");
    assert_ne!(before, after, "restart should replace the process");
}
