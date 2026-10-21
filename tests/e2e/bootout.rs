use crate::harness::{self, TempServiceBuilder};

#[test]
fn bootout_unloads_service() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("bootout").build();
    svc.install_via_lctl();
    harness::assert_loaded(&svc.target);

    harness::lctl()
        .args(["bootout", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("booted out {}", svc.label)));

    harness::assert_not_loaded(&svc.target);
    // leave plist for Drop cleanup of file; service already unloaded
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_bootout_unloads_service() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("bootout").build();
    svc.install_via_lctl();
    harness::assert_loaded_sudo(&svc.target);

    harness::lctl_sudo()
        .args(["bootout", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("booted out {}", svc.label)));

    harness::assert_not_loaded_sudo(&svc.target);
}
