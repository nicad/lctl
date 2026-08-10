use crate::harness::{self, TempServiceBuilder};

#[test]
fn install_symlinks_plist_and_loads_service() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("install").build();

    harness::angel()
        .args([
            "install",
            "--domain",
            "gui",
            svc.fixture_plist.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("installed {}", svc.label)));

    harness::assert_installed_symlink(&svc);
    harness::assert_loaded(&svc.target);
    harness::assert_not_running(&svc.target);
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_install_symlinks_plist_sets_root_ownership_and_loads() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("install").build();

    harness::angel_sudo()
        .args([
            "install",
            "--domain",
            "system",
            svc.fixture_plist.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("installed {}", svc.label)));

    harness::assert_installed_symlink(&svc);
    harness::assert_system_install_permissions(&svc);
    harness::assert_loaded_sudo(&svc.target);
    harness::assert_not_running_sudo(&svc.target);
}
