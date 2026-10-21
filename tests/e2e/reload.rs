use crate::fixtures::ProgramKind;
use crate::harness::{self, TempServiceBuilder};

#[test]
fn reload_reloads_from_disk() {
    harness::require_macos();
    // Default install = symlink; rewrite fixture target then reload must pick up new program.
    let svc = TempServiceBuilder::gui("reload").build();
    svc.install_via_lctl();
    harness::assert_loaded(&svc.target);

    let before = harness::print_service(&svc.target).expect("print before reload");
    assert!(
        before.contains("/usr/bin/true"),
        "expected initial program /usr/bin/true in launchctl print"
    );

    svc.rewrite_fixture(ProgramKind::Sleep, false);

    harness::lctl()
        .args(["reload", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("reloaded {}", svc.label)));

    harness::assert_loaded(&svc.target);
    let after = harness::print_service(&svc.target).expect("print after reload");
    assert!(
        after.contains("/bin/sleep"),
        "reload should reload symlink target; expected /bin/sleep in launchctl print, got:\n{after}"
    );
    assert!(!after.contains("/usr/bin/true"), "old program should be gone after reload");
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_reload_reloads_from_disk() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("reload").build();
    svc.install_via_lctl();
    harness::assert_loaded_sudo(&svc.target);

    let before = harness::print_service_sudo(&svc.target).expect("print before reload");
    assert!(
        before.contains("/usr/bin/true"),
        "expected initial program /usr/bin/true in launchctl print"
    );

    // Fixture is root-owned after system install; rewrite via sudo tee.
    svc.rewrite_fixture(ProgramKind::Sleep, false);

    harness::lctl_sudo()
        .args(["reload", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("reloaded {}", svc.label)));

    harness::assert_loaded_sudo(&svc.target);
    let after = harness::print_service_sudo(&svc.target).expect("print after reload");
    assert!(
        after.contains("/bin/sleep"),
        "reload should reload symlink target; expected /bin/sleep in launchctl print, got:\n{after}"
    );
    assert!(!after.contains("/usr/bin/true"), "old program should be gone after reload");
}
