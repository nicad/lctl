use crate::fixtures::ProgramKind;
use crate::harness::{self, TempServiceBuilder};

#[test]
fn reboot_reloads_from_disk() {
    harness::require_macos();
    // Default install = symlink; rewrite fixture target then reboot must pick up new program.
    let svc = TempServiceBuilder::gui("reboot").build();
    svc.install_via_angel();
    harness::assert_loaded(&svc.target);

    let before = harness::print_service(&svc.target).expect("print before reboot");
    assert!(
        before.contains("/usr/bin/true"),
        "expected initial program /usr/bin/true in launchctl print"
    );

    svc.rewrite_fixture(ProgramKind::Sleep, false);

    harness::angel()
        .args(["reboot", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("rebooted {}", svc.label)));

    harness::assert_loaded(&svc.target);
    let after = harness::print_service(&svc.target).expect("print after reboot");
    assert!(
        after.contains("/bin/sleep"),
        "reboot should reload symlink target; expected /bin/sleep in launchctl print, got:\n{after}"
    );
    assert!(!after.contains("/usr/bin/true"), "old program should be gone after reboot");
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_reboot_reloads_from_disk() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("reboot").build();
    svc.install_via_angel();
    harness::assert_loaded_sudo(&svc.target);

    let before = harness::print_service_sudo(&svc.target).expect("print before reboot");
    assert!(
        before.contains("/usr/bin/true"),
        "expected initial program /usr/bin/true in launchctl print"
    );

    // Fixture is root-owned after system install; rewrite via sudo tee.
    svc.rewrite_fixture(ProgramKind::Sleep, false);

    harness::angel_sudo()
        .args(["reboot", "-e", &svc.label])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("rebooted {}", svc.label)));

    harness::assert_loaded_sudo(&svc.target);
    let after = harness::print_service_sudo(&svc.target).expect("print after reboot");
    assert!(
        after.contains("/bin/sleep"),
        "reboot should reload symlink target; expected /bin/sleep in launchctl print, got:\n{after}"
    );
    assert!(!after.contains("/usr/bin/true"), "old program should be gone after reboot");
}
