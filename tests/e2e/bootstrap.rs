use crate::harness::{self, TempServiceBuilder};

#[test]
fn bootstrap_loads_plist_from_disk() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("bootstrap").build();
    // Place plist without loading, then bootout any leftover, then lctl bootstrap.
    std::fs::create_dir_all(harness::agents_dir()).ok();
    std::fs::copy(&svc.fixture_plist, &svc.installed_plist).expect("place plist");
    let _ = harness::launchctl(&["bootout", &svc.target]);

    harness::lctl()
        .args(["bootstrap", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("bootstrapped {}", svc.label)));

    harness::assert_loaded(&svc.target);
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_bootstrap_loads_plist_from_disk() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("bootstrap").build();
    svc.place_system_plist_unloaded();

    harness::lctl_sudo()
        .args(["bootstrap", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("bootstrapped {}", svc.label)));

    harness::assert_loaded_sudo(&svc.target);
}
