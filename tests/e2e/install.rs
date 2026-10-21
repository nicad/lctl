use crate::harness::{self, TempServiceBuilder};

#[test]
fn install_symlinks_plist_and_loads_service() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("install").build();

    harness::lctl()
        .args(["install", "--domain", "gui", svc.fixture_plist.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("installed {}", svc.label)));

    harness::assert_installed_symlink(&svc);
    harness::assert_loaded(&svc.target);
    harness::assert_not_running(&svc.target);
}

/// Plists under `{LCTL_CONFIG_DIR}/gui` (default `~/Library/Application Support/lctl/gui`).
/// Fuzzy query matches multiple → select (default / `-y` picks lexicographically first).
#[test]
fn install_by_fuzzy_name_from_lctl_gui() {
    harness::require_macos();
    let mut first = TempServiceBuilder::gui("install-fuzzy-a").build();
    let mut second = TempServiceBuilder::gui("install-fuzzy-b").build();

    let lctl_config = first.temp.path().join("lctl-config");
    let gui_dir = lctl_config.join("gui");
    std::fs::create_dir_all(&gui_dir).expect("mkdir lctl gui");

    for svc in [&mut first, &mut second] {
        let dest = gui_dir.join(format!("{}.plist", svc.label));
        std::fs::rename(&svc.fixture_plist, &dest).expect("move plist into lctl gui");
        svc.fixture_plist = dest;
    }

    // Shared substring; both lctl-gui plists match. `-y` takes sorted default (first).
    let query = "install-fuzzy";
    assert!(first.label.contains(query) && second.label.contains(query));
    let (expected, other) =
        if first.label < second.label { (&first, &second) } else { (&second, &first) };

    harness::lctl()
        .env("LCTL_CONFIG_DIR", &lctl_config)
        .args(["install", query])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("installed {}", expected.label)));

    harness::assert_installed_symlink(expected);
    harness::assert_loaded(&expected.target);
    harness::assert_not_running(&expected.target);
    assert!(
        !other.installed_plist.exists()
            && std::fs::symlink_metadata(&other.installed_plist).is_err(),
        "non-selected match should not be installed"
    );
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_install_symlinks_plist_sets_root_ownership_and_loads() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("install").build();

    harness::lctl_sudo()
        .args(["install", "--domain", "system", svc.fixture_plist.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("installed {}", svc.label)));

    harness::assert_installed_symlink(&svc);
    harness::assert_system_install_permissions(&svc);
    harness::assert_loaded_sudo(&svc.target);
    harness::assert_not_running_sudo(&svc.target);
}
