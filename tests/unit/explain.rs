use clap::CommandFactory;
use lctl::cli::Cli;
use lctl::explain::{about, everything};

#[test]
fn everything_renders_the_readme_section_without_markdown_syntax() {
    let text = everything();
    assert!(text.contains("How lctl maps to launchctl"));
    assert!(text.contains("launchctl bootstrap <domain> <plist>"));
    for leftover in ["<br>", "](#", "<!--", "|---", "**"] {
        assert!(!text.contains(leftover), "{leftover:?} left in the output");
    }
    // stops at the end marker, before the Configuration section
    assert!(!text.contains("lctl optionally reads YAML"));
}

// Tables wrap to the terminal width when stdout is a terminal (as under `make test`), so
// checks on table cells look for single words, which only split in a very narrow terminal.

#[test]
fn about_an_lctl_command_shows_its_row() {
    let text = about("stop").unwrap();
    assert!(text.contains("bootout"));
    assert!(!text.contains("kickstart"), "restart's row should not be included");
}

#[test]
fn about_a_launchctl_verb_shows_the_lctl_equivalent() {
    let text = about("load").unwrap();
    assert!(text.contains("bootstrap"), "modern equivalent");
    assert!(text.contains("install"), "lctl equivalent");
}

#[test]
fn about_includes_subsections_titled_after_the_command() {
    assert!(about("install").unwrap().contains("chown -h root:wheel"));
    assert!(about("doctor").unwrap().contains("LimitLoadToSessionType"));
}

#[test]
fn about_ignores_case_and_returns_none_for_unknown_commands() {
    assert!(about("STOP").is_some());
    assert!(about("no-such-command").is_none());
}

#[test]
fn every_subcommand_help_points_to_an_explanation() {
    for subcommand in Cli::command().get_subcommands() {
        let name = subcommand.get_name();
        if name != "explain" {
            assert!(about(name).is_some(), "`lctl explain {name}` has nothing to show");
        }
    }
}
