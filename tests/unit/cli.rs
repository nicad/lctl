use clap::Parser;
use lctl::cli::{Cli, Commands};

fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(std::iter::once("lctl").chain(args.iter().copied()))
}

#[test]
fn all_flag_is_accepted_by_commands_that_act_on_several_services() {
    for command in
        ["start", "stop", "restart", "reload", "enable", "disable", "kill", "status", "ls"]
    {
        assert!(parse(&[command, "-A", "foo"]).is_ok(), "{command} -A");
    }
    assert!(parse(&["doctor", "--all", "foo"]).is_ok());
    let Some(Commands::Restart(args)) = parse(&["restart", "-A", "foo"]).unwrap().command else {
        panic!("expected restart");
    };
    assert!(args.all && args.name == "foo");
}

#[test]
fn all_flag_needs_a_name_where_the_name_is_optional() {
    assert!(parse(&["status", "-A"]).is_err());
    assert!(parse(&["doctor", "-A"]).is_err());
}

#[test]
fn list_all_flag_rejects_filters_that_other_commands_do_not_apply() {
    for filter in ["-a", "-d", "--running", "--failed", "--disabled"] {
        assert!(parse(&["ls", "-A", filter, "foo"]).is_err(), "ls -A {filter}");
    }
    assert!(parse(&["ls", "-A", "-l", "-e", "foo"]).is_ok());
}

#[test]
fn all_flag_is_rejected_where_one_service_is_expected() {
    for command in ["uninstall", "logs", "plist"] {
        assert!(parse(&[command, "-A", "foo"]).is_err(), "{command} -A");
    }
}
