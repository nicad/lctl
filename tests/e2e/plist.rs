use crate::harness::{self, TempServiceBuilder};

#[test]
fn plist_dumps_installed_file() {
    harness::require_macos();
    let svc = TempServiceBuilder::gui("plist").build();
    svc.install_via_angel();

    let on_disk = std::fs::read_to_string(&svc.installed_plist).expect("read installed plist");

    let output =
        harness::angel().args(["plist", "-e", &svc.label]).output().expect("run angel plist");
    assert!(output.status.success(), "plist command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&svc.label));
    assert!(stdout.contains("LimitLoadToSessionType"));
    assert_eq!(stdout, on_disk);
}

#[test]
#[ignore = "system domain; run: sudo -v && cargo e2e-all"]
fn system_plist_dumps_installed_file() {
    harness::require_root_e2e();
    let svc = TempServiceBuilder::system("plist").build();
    svc.install_via_angel();

    let on_disk = std::fs::read_to_string(&svc.installed_plist).expect("read installed plist");

    let output =
        harness::angel_sudo().args(["plist", "-e", &svc.label]).output().expect("run angel plist");
    assert!(output.status.success(), "plist command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&svc.label));
    assert!(stdout.contains("LimitLoadToSessionType"));
    assert!(stdout.contains("System"));
    assert_eq!(stdout, on_disk);
}
