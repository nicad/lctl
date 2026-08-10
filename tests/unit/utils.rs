use angel::Error;
use angel::types::{Daemon, Domain, ForWhom};
use angel::utils::{expand_path, home, pick_daemon};
use std::path::PathBuf;

#[test]
fn expand_path_leaves_absolute_unchanged() {
    let p = expand_path("/tmp/foo").unwrap();
    assert_eq!(p, PathBuf::from("/tmp/foo"));
}

#[test]
fn expand_path_expands_tilde_slash() {
    let home_path = home().unwrap().clone();
    let p = expand_path("~/Library/LaunchAgents").unwrap();
    assert_eq!(p, home_path.join("Library/LaunchAgents"));
}

#[test]
fn expand_path_bare_tilde_is_home() {
    let home_path = home().unwrap().clone();
    assert_eq!(expand_path("~").unwrap(), home_path);
}

#[test]
fn pick_daemon_not_found() {
    let err = pick_daemon(vec![], "missing").unwrap_err();
    assert!(matches!(err, Error::NotFound(_)));
}

#[test]
fn pick_daemon_single_match() {
    let daemon = Daemon {
        name: "com.example.one".into(),
        source_path: None,
        domain: Domain::Gui(501),
        for_use_by: ForWhom::User,
        plist: None,
        pid: None,
        last_exit_code: None,
    };
    let picked = pick_daemon(vec![&daemon], "one").unwrap();
    assert_eq!(picked.name, "com.example.one");
}
