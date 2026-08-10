use angel::parser::{ServiceItem, parse_print_service};

#[test]
fn parses_flat_service_fields() {
    let output = "\
com.example.svc = {
\tstate = running
\tpid = 4242
\tpath = /tmp/com.example.svc.plist
}
";
    let parsed = parse_print_service(output).unwrap();
    assert_eq!(parsed.get("state"), Some(&ServiceItem::Str("running")));
    assert_eq!(parsed.get("pid"), Some(&ServiceItem::Str("4242")));
    assert_eq!(
        parsed.get("path"),
        Some(&ServiceItem::Str("/tmp/com.example.svc.plist"))
    );
}

#[test]
fn parses_nested_array_and_dict() {
    let output = "\
com.example.svc = {
\targuments = {
\t\t/usr/bin/foo
\t\t--flag
\t}
\tenvironment = {
\t\t\"PATH\" => /usr/bin
\t}
}
";
    let parsed = parse_print_service(output).unwrap();
    match parsed.get("arguments") {
        Some(ServiceItem::Array(items)) => {
            assert_eq!(
                items,
                &vec![ServiceItem::Str("/usr/bin/foo"), ServiceItem::Str("--flag")]
            );
        }
        other => panic!("expected arguments array, got {other:?}"),
    }
    match parsed.get("environment") {
        Some(ServiceItem::Dict(items)) => {
            assert_eq!(
                items,
                &vec![ServiceItem::KeyValue("PATH", Box::new(ServiceItem::Str("/usr/bin")))]
            );
        }
        other => panic!("expected environment dict, got {other:?}"),
    }
}

#[test]
fn rejects_non_dict_root() {
    assert!(parse_print_service("not a dict\n").is_err());
}

#[test]
fn parses_state_not_running_without_pid() {
    let output = "\
com.example.idle = {
\tstate = not running
\tpath = /tmp/idle.plist
}
";
    let parsed = parse_print_service(output).unwrap();
    assert_eq!(parsed.get("state"), Some(&ServiceItem::Str("not running")));
    assert!(parsed.get("pid").is_none());
}
