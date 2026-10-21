use lctl::describe::runs_when;
use lctl::launchctl::LaunchctlOutput;
use lctl::types::{Daemon, Domain, ForWhom, KeepAlive, Plist};
use plist::{Dictionary, Value};

fn calendar(fields: &[(&str, i64)]) -> Value {
    let mut entry = Dictionary::new();
    for (key, value) in fields {
        entry.insert(key.to_string(), Value::Integer((*value).into()));
    }
    Value::Dictionary(entry)
}

fn when(plist: Plist) -> String {
    runs_when(&plist)
}

#[test]
fn runs_when_without_triggers_is_manual_only() {
    assert_eq!(when(Plist::default()), "manual only");
}

#[test]
fn runs_when_lists_load_and_keep_alive() {
    let plist = Plist {
        run_at_load: Some(true),
        keep_alive: Some(KeepAlive::Boolean(true)),
        ..Default::default()
    };
    assert_eq!(when(plist), "at load, keep-alive");
}

#[test]
fn runs_when_formats_intervals() {
    let every = |seconds| when(Plist { start_interval: Some(seconds), ..Default::default() });
    assert_eq!(every(300), "every 5m");
    assert_eq!(every(3600), "every 1h");
    assert_eq!(every(86400), "every 1d");
    assert_eq!(every(90), "every 90s");
}

#[test]
fn runs_when_formats_calendar_entries() {
    let at = |value| when(Plist { start_calendar_interval: Some(value), ..Default::default() });
    assert_eq!(at(calendar(&[("Hour", 2), ("Minute", 0)])), "daily 02:00");
    assert_eq!(at(calendar(&[("Weekday", 1), ("Hour", 9), ("Minute", 30)])), "Mon 09:30");
    assert_eq!(at(calendar(&[("Minute", 15)])), "hourly at :15");
    assert_eq!(at(calendar(&[("Day", 1), ("Hour", 12), ("Minute", 0)])), "day 1 12:00");
    assert_eq!(at(Value::Array(vec![calendar(&[("Minute", 0)]); 3])), "3 calendar times");
}

#[test]
fn runs_when_reports_on_demand_services() {
    let plist =
        Plist { mach_services: Some(Value::Dictionary(Dictionary::new())), ..Default::default() };
    assert_eq!(when(plist), "on demand");
}

fn daemon(loaded: bool, disabled: bool) -> Daemon {
    let plist = Plist { label: Some("com.example.svc".into()), ..Default::default() };
    let mut daemon = Daemon::from_plist(plist, None, Domain::Gui(501), ForWhom::User, 501);
    daemon.loaded = loaded;
    daemon.disabled = disabled;
    daemon
}

fn failed(stderr: &str) -> LaunchctlOutput {
    LaunchctlOutput { stdout: String::new(), stderr: stderr.into(), exit_code: Some(1) }
}

#[test]
fn failure_hint_narrows_input_output_error_with_known_state() {
    let io_error = failed("Bootstrap failed: 5: Input/output error\n");
    assert!(io_error.failure_hint(&daemon(true, false)).unwrap().contains("lctl reload"));
    assert!(io_error.failure_hint(&daemon(false, false)).unwrap().contains("lctl doctor"));

    let mut user_domain = daemon(false, false);
    user_domain.domain = Domain::User(501);
    assert!(io_error.failure_hint(&user_domain).unwrap().contains("belong in gui"));
}

#[test]
fn failure_hint_for_operation_not_permitted_says_root() {
    let output = failed("Boot-out failed: 1: Operation not permitted\n");
    assert!(output.failure_hint(&daemon(true, false)).unwrap().contains("needs root"));
}

#[test]
fn failure_hint_for_missing_service_suggests_start() {
    let output = failed("Could not find service \"com.example.svc\" in domain for user gui: 501\n");
    let hint = output.failure_hint(&daemon(false, false)).unwrap();
    assert!(hint.contains("lctl start com.example.svc"), "{hint}");
}

#[test]
fn failure_hint_for_sip_says_it_cannot_be_changed() {
    let output = failed(
        "Boot-out failed: 150: Operation not permitted while System Integrity Protection is engaged\n",
    );
    let hint = output.failure_hint(&daemon(true, false)).unwrap();
    assert!(hint.contains("System Integrity Protection"), "{hint}");
}

#[test]
fn failure_hint_is_none_for_unknown_errors() {
    assert!(failed("something else\n").failure_hint(&daemon(true, false)).is_none());
}
