use crate::harness::{self, TempServiceBuilder};
use predicates::prelude::*;

#[test]
fn doctor_passes_for_healthy_agent() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("doctor").echo_then_sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running(&svc.target);

    // Also guards against false positives: log access and loaded-vs-disk comparison.
    harness::lctl()
        .args(["doctor", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(&svc.label))
        .stdout(predicates::str::contains("not writable").not())
        .stdout(predicates::str::contains("changed on disk").not())
        .stdout(predicates::str::contains("✗").not());
}

#[test]
fn doctor_fails_for_missing_program() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("doctor-missing").missing_program().build();
    svc.place_gui_plist_unloaded();

    harness::lctl()
        .args(["doctor", "-e", &svc.label])
        .assert()
        .failure()
        .stdout(predicates::str::contains("program not found: /nonexistent/lctl-e2e-missing"));
}
