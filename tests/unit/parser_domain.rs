use lctl::parser::{DomainService, parse_domain_output};

#[test]
fn parses_services_block() {
    let input = br#"gui/501 = {
	type = prompt
	services = {
		123	0	com.example.running
		-	78	com.example.idle
	}
	disabled services = {
	}
}
"#;
    let services = parse_domain_output(input).unwrap();
    assert_eq!(services.len(), 2);
    assert_eq!(
        services[0],
        DomainService {
            pid: Some(123),
            last_exit_code: Some("0".into()),
            name: "com.example.running".into(),
        }
    );
    assert_eq!(
        services[1],
        DomainService {
            pid: None,
            last_exit_code: Some("78".into()),
            name: "com.example.idle".into(),
        }
    );
}

#[test]
fn zero_pid_means_not_running() {
    let input = br#"system = {
	services = {
		       0      - 	com.example.stopped
	}
}
"#;
    let services = parse_domain_output(input).unwrap();
    assert_eq!(services[0].pid, None);
}

#[test]
fn rejects_missing_services_block() {
    let input = b"gui/501 = {\n\ttype = prompt\n}\n";
    assert!(parse_domain_output(input).is_err());
}

#[test]
fn parses_empty_services_block() {
    let input = br#"user/501 = {
	services = {
	}
}
"#;
    let services = parse_domain_output(input).unwrap();
    assert!(services.is_empty());
}
