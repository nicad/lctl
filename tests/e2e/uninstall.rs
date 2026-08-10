use crate::harness::{self, TempServiceBuilder};

#[test]
fn uninstall_removes_plist_and_unloads() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("uninstall").build();
    svc.install_via_angel();

    harness::angel()
        .args(["uninstall", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains("Uninstalled"));

    harness::assert_not_loaded(&svc.target);
    assert!(!svc.installed_plist.exists(), "installed plist should be removed");
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_uninstall_removes_plist_and_unloads() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("uninstall").build();
    svc.install_via_angel();

    harness::angel_sudo()
        .args(["uninstall", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains("Uninstalled"));

    harness::assert_not_loaded_sudo(&svc.target);
    assert!(
        std::fs::symlink_metadata(&svc.installed_plist).is_err(),
        "system plist symlink should be removed"
    );
}
