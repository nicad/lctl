use lctl::types::{Daemon, Domain, ForWhom, Plist};
use std::path::PathBuf;

fn plist_session(session: &str) -> Plist {
    Plist { limit_load_to_session_type: Some(session.to_string()), ..Default::default() }
}

#[test]
fn domain_from_plist_session_types() {
    let uid = 501;
    assert_eq!(Domain::from_plist(&plist_session("Aqua"), uid, Domain::System), Domain::Gui(uid));
    assert_eq!(
        Domain::from_plist(&plist_session("Background"), uid, Domain::System),
        Domain::User(uid)
    );
    assert_eq!(
        Domain::from_plist(&plist_session("LoginWindow"), uid, Domain::System),
        Domain::User(uid)
    );
    assert_eq!(Domain::from_plist(&plist_session("System"), uid, Domain::Gui(uid)), Domain::System);
}

#[test]
fn domain_from_plist_unknown_or_missing_uses_default() {
    let uid = 501;
    let default = Domain::User(uid);
    assert_eq!(Domain::from_plist(&plist_session("SomethingElse"), uid, default), default);
    assert_eq!(Domain::from_plist(&Plist::default(), uid, default), default);
}

#[test]
fn domain_from_plist_missing_session_uses_gui_launch_agents_default() {
    let uid = 501;
    let launch_agents_default = Domain::Gui(uid);
    assert_eq!(Domain::from_plist(&Plist::default(), uid, launch_agents_default), Domain::Gui(uid));
    assert_eq!(
        Domain::from_plist(&plist_session("SomethingElse"), uid, launch_agents_default),
        Domain::Gui(uid)
    );
}

#[test]
fn domain_display() {
    assert_eq!(Domain::System.to_string(), "system");
    assert_eq!(Domain::User(501).to_string(), "user/501");
    assert_eq!(Domain::Gui(501).to_string(), "gui/501");
    assert_eq!(Domain::Unknown.to_string(), "Unknown");
}

#[test]
fn service_target_formats_domain_and_name() {
    let daemon = Daemon {
        name: "com.example.svc".into(),
        source_path: None,
        domain: Domain::Gui(501),
        for_use_by: ForWhom::User,
        plist: None,
        pid: None,
        last_exit_code: None,
        loaded: false,
        disabled: false,
    };
    assert_eq!(daemon.service_target(), "gui/501/com.example.svc");
}

#[test]
fn from_plist_uses_label_and_session() {
    let plist = Plist {
        label: Some("com.example.fromplist".into()),
        limit_load_to_session_type: Some("Aqua".into()),
        ..Default::default()
    };
    let daemon = Daemon::from_plist(
        plist,
        Some(PathBuf::from("/tmp/com.example.fromplist.plist")),
        Domain::System,
        ForWhom::Lctl,
        501,
    );
    assert_eq!(daemon.name, "com.example.fromplist");
    assert_eq!(daemon.domain, Domain::Gui(501));
    assert_eq!(daemon.for_use_by, ForWhom::Lctl);
}

#[test]
fn require_source_path_errors_when_missing() {
    let daemon = Daemon {
        name: "x".into(),
        source_path: None,
        domain: Domain::System,
        for_use_by: ForWhom::User,
        plist: None,
        pid: None,
        last_exit_code: None,
        loaded: false,
        disabled: false,
    };
    assert!(daemon.require_source_path().is_err());
}

#[test]
fn state_from_launchd_fields() {
    use lctl::types::ServiceState;
    let mut daemon =
        Daemon::from_plist(Plist::default(), None, Domain::Gui(501), ForWhom::User, 501);
    assert_eq!(daemon.state(), ServiceState::NotLoaded);

    daemon.disabled = true;
    assert_eq!(daemon.state(), ServiceState::Disabled);

    daemon.loaded = true;
    for idle_code in ["-", "0", "(jt)"] {
        daemon.last_exit_code = Some(idle_code.into());
        assert_eq!(daemon.state(), ServiceState::Idle, "exit code {idle_code}");
    }
    daemon.last_exit_code = Some("78".into());
    assert_eq!(daemon.state(), ServiceState::Failed(78));

    daemon.pid = Some(42);
    assert_eq!(daemon.state(), ServiceState::Running);
}
