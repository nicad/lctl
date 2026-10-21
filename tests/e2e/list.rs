use crate::harness::{self, TempServiceBuilder};

#[test]
fn list_includes_running_service() {
    harness::require_macos();
    // Default list hides idle (no pid) unless `-i`; keep a live sleep job.
    let svc = TempServiceBuilder::gui("list").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running(&svc.target);

    harness::lctl()
        .args(["list", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(&svc.label));
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_list_includes_running_service() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("list").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running_sudo(&svc.target);

    harness::lctl_sudo()
        .args(["list", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(&svc.label))
        .stdout(predicates::str::contains("system"));
}
