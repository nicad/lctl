use lctl::process::start_time;
use std::time::{Duration, SystemTime};

#[test]
fn start_time_of_this_process_is_recent() {
    let start = start_time(std::process::id()).expect("start time of the test process");
    let age = SystemTime::now().duration_since(start).expect("started in the past");
    assert!(age < Duration::from_secs(3600), "{age:?}");
}

#[test]
fn start_time_of_launchd_is_available_without_root() {
    assert!(start_time(1).is_some());
}

#[test]
fn start_time_of_a_missing_process_is_none() {
    assert_eq!(start_time(99_999_999), None);
}
