use crate::harness::{self, TempServiceBuilder};

#[test]
fn restart_all_restarts_every_match() {
    harness::require_macos();
    let services = [
        TempServiceBuilder::gui("all-restart").sleep().run_at_load().build(),
        TempServiceBuilder::gui("all-restart").sleep().run_at_load().build(),
    ];
    let mut before = Vec::new();
    for svc in &services {
        svc.install_via_lctl();
        harness::assert_running(&svc.target);
        before.push(harness::pid(&svc.target).expect("pid before restart"));
    }

    let mut assert =
        harness::lctl().args(["restart", "-A", &harness::label_query("all-restart")]).assert();
    for svc in &services {
        assert = assert.stdout(predicates::str::contains(format!("restarted {}", svc.label)));
    }
    assert.success();

    for (svc, before) in services.iter().zip(before) {
        harness::assert_running(&svc.target);
        let after = harness::pid(&svc.target).expect("pid after restart");
        assert_ne!(before, after, "{} should have a new process", svc.label);
    }
}

#[test]
fn kill_all_continues_after_a_failure() {
    harness::require_macos();
    let running = TempServiceBuilder::gui("all-kill").sleep().run_at_load().build();
    let idle = TempServiceBuilder::gui("all-kill").sleep().build();
    running.install_via_lctl();
    idle.install_via_lctl();
    harness::assert_running(&running.target);

    harness::lctl()
        .args(["kill", "-A", &harness::label_query("all-kill")])
        .assert()
        .failure()
        .stdout(predicates::str::contains(format!("sent TERM to {}", running.label)))
        .stderr(predicates::str::contains(format!("{} is not running (idle)", idle.label)))
        .stderr(predicates::str::contains("1 of 2 services failed"));

    harness::assert_not_running(&running.target);
}

#[test]
fn status_all_shows_every_match() {
    harness::require_macos();
    let first = TempServiceBuilder::gui("all-status").build();
    let second = TempServiceBuilder::gui("all-status").build();
    first.place_gui_plist_unloaded();
    second.place_gui_plist_unloaded();

    harness::lctl()
        .args(["status", "-A", &harness::label_query("all-status")])
        .assert()
        .success()
        .stdout(predicates::str::contains(&first.label))
        .stdout(predicates::str::contains(&second.label));
}

#[test]
fn change_all_without_terminal_or_yes_does_nothing() {
    harness::require_macos();
    let first = TempServiceBuilder::gui("all-confirm").build();
    let second = TempServiceBuilder::gui("all-confirm").build();
    first.install_via_lctl();
    second.install_via_lctl();

    harness::lctl_without_yes()
        .args(["stop", "-A", &harness::label_query("all-confirm")])
        .assert()
        .failure()
        .stderr(predicates::str::contains("add -y"));

    assert!(harness::is_loaded(&first.target) && harness::is_loaded(&second.target));
}
