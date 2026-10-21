use crate::harness::{self, TempServiceBuilder};

#[test]
fn print_shows_launchctl_output() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("print").build();
    svc.install_via_lctl();

    harness::lctl()
        .args(["print", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(&svc.label));
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_print_shows_launchctl_output() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("print").build();
    svc.install_via_lctl();

    harness::lctl_sudo()
        .args(["print", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(&svc.label))
        .stdout(predicates::str::contains("system"));
}
