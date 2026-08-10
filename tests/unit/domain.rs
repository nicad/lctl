use angel::types::{Daemon, Domain, ForWhom, Plist};
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
        ForWhom::Angel,
        501,
    );
    assert_eq!(daemon.name, "com.example.fromplist");
    assert_eq!(daemon.domain, Domain::Gui(501));
    assert_eq!(daemon.for_use_by, ForWhom::Angel);
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
    };
    assert!(daemon.require_source_path().is_err());
}
