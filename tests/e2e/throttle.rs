use crate::harness::{self, TempServiceBuilder};
use std::time::Duration;

fn throttled(target: &str) -> bool {
    harness::print_service(target).is_some_and(|print| print.contains("state = spawn scheduled"))
}

#[test]
fn list_and_status_show_a_throttled_service() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("throttle").exits_kept_alive().build();
    svc.install_via_lctl();
    assert!(
        harness::wait_until(|| throttled(&svc.target), Duration::from_secs(5)),
        "expected launchd to throttle {}",
        svc.label
    );

    harness::lctl()
        .args(["list", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains("throttled"));

    harness::lctl()
        .args(["status", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains("throttled"))
        .stdout(predicates::str::contains(format!("lctl reload {}", svc.label)));
}
