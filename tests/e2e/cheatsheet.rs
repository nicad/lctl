use crate::harness;

#[test]
fn cheatsheet_domains_prints_reference() {
    harness::require_macos();

    harness::angel()
        .args(["cheatsheet", "domains"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Launchd Domains"))
        .stdout(predicates::str::contains("System Domain"))
        .stdout(predicates::str::contains("GUI Domain"));
}
