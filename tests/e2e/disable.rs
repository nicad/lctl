use crate::harness::{self, TempServiceBuilder};

#[test]
fn disable_marks_service_disabled_in_domain() {
    harness::require_macos();
    let mut svc = TempServiceBuilder::gui("disable").sleep().build();
    svc.install_via_lctl();

    harness::lctl()
        .args(["disable", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("disabled {}", svc.label)));
    svc.mark_disabled();

    // macOS still allows kickstart after disable; the durable signal is the domain map.
    harness::assert_disabled(&svc.label);
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_disable_marks_service_disabled_in_domain() {
    harness::require_root_e2e();
    let mut svc = TempServiceBuilder::system("disable").sleep().build();
    svc.install_via_lctl();

    harness::lctl_sudo()
        .args(["disable", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("disabled {}", svc.label)));
    svc.mark_disabled();

    harness::assert_disabled_system(&svc.label);
}
