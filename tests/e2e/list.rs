use crate::harness::{self, TempServiceBuilder};
use predicates::prelude::*;

#[test]
fn list_includes_running_service() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("list").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running(&svc.target);

    harness::lctl()
        .args(["list", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(&svc.label))
        .stdout(predicates::str::contains("Started"))
        .stdout(predicates::str::is_match(r"\d{2}:\d{2} +\d+[smhd] ").unwrap())
        .stdout(predicates::str::contains("Domain").not());

    harness::lctl()
        .args(["list", "-l", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("gui/{}", harness::uid())))
        .stdout(predicates::str::contains(format!("{}.plist", svc.label)));
}

#[test]
fn bare_lctl_shows_overview_with_idle_service() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("overview").build();
    svc.install_via_lctl();

    harness::lctl()
        .assert()
        .success()
        .stdout(predicates::str::contains(&svc.label))
        .stdout(predicates::str::contains("Runs when"))
        .stdout(predicates::str::contains("manual only"));
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_list_includes_running_service() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("list").sleep().run_at_load().build();
    svc.install_via_lctl();
    harness::assert_running_sudo(&svc.target);

    harness::lctl_sudo()
        .args(["list", "-l", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(&svc.label))
        .stdout(predicates::str::contains("system"));
}
