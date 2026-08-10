use crate::harness::{self, TempServiceBuilder};

/// Clear the sudo timestamp, then run plain `angel` (not `sudo angel`) with a real TTY so
/// auto-elevate prompts and completes a system install.
#[test]
#[ignore = "system domain; interactive sudo — run: sudo -v && cargo e2e-all"]
fn system_install_after_sudo_k_prompts_and_succeeds() {
    harness::require_interactive_sudo_e2e();
    // Start from a warm ticket (outer `sudo -v`), then invalidate so angel must prompt.
    harness::require_root_e2e();

    let svc = TempServiceBuilder::system("install-sudo-prompt").build();

    harness::invalidate_sudo_ticket();
    assert!(!harness::sudo_n_ok(), "sudo -K should clear the cached ticket");

    eprintln!(
        "\n=== angel will prompt for your password (interactive auto-elevate) — enter it ===\n"
    );

    let status = harness::angel_tty_status(&[
        "install",
        "--domain",
        "system",
        svc.fixture_plist.to_str().unwrap(),
    ]);
    assert!(status.success(), "angel system install should succeed after interactive sudo");

    // Ticket should be warm again for later system e2e.
    assert!(harness::sudo_n_ok(), "interactive sudo should leave a warm ticket");

    harness::assert_installed_symlink(&svc);
    harness::assert_system_install_permissions(&svc);
    harness::assert_loaded_sudo(&svc.target);
    harness::assert_not_running_sudo(&svc.target);
}

/// Same clear-and-prompt path, but via launchctl retry (bootout) rather than install FS ops.
#[test]
#[ignore = "system domain; interactive sudo — run: sudo -v && cargo e2e-all"]
fn system_bootout_after_sudo_k_prompts_and_succeeds() {
    harness::require_interactive_sudo_e2e();
    harness::require_root_e2e();

    let svc = TempServiceBuilder::system("bootout-sudo-prompt").build();
    svc.install_via_angel();
    harness::assert_loaded_sudo(&svc.target);

    harness::invalidate_sudo_ticket();
    assert!(!harness::sudo_n_ok(), "sudo -K should clear the cached ticket");

    eprintln!(
        "\n=== angel will prompt for your password (launchctl auto-elevate) — enter it ===\n"
    );

    let status = harness::angel_tty_status(&["bootout", "-e", &svc.label]);
    assert!(status.success(), "angel system bootout should succeed after interactive sudo");

    assert!(harness::sudo_n_ok(), "interactive sudo should leave a warm ticket");
    harness::assert_not_loaded_sudo(&svc.target);
}
