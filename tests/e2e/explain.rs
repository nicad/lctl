use crate::harness;

#[test]
fn explain_shows_launchctl_calls_and_help_points_to_it() {
    harness::lctl()
        .args(["explain", "stop"])
        .assert()
        .success()
        .stdout(predicates::str::contains("launchctl bootout <target>"));

    harness::lctl()
        .args(["stop", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Under the hood: lctl explain stop"));

    harness::lctl().args(["explain", "no-such-command"]).assert().failure();
}
