use crate::cli::{ListArgs, OptionalNameArgs};
use crate::doctor::{self, Check, Severity};
use crate::error::{Error, Result};
use crate::lctl::Lctl;
use crate::output::{self, stdout};
use crate::parser;
use crate::styles;
use crate::types::Daemon;
use crossterm::style::Color;

pub(crate) fn run(lctl: &Lctl, args: &OptionalNameArgs) -> Result<()> {
    let daemons = match &args.name {
        Some(name) => super::resolve_daemons(lctl, name, args.exact, args.all)?,
        None => super::list::listed(lctl, &ListArgs::default()),
    };
    // several services: report only those with problems, then a count
    let sweep = args.name.is_none() || args.all;
    let verbose = output::is_verbose();

    let (mut failed_checks, mut failing, mut warning) = (0, 0, 0);
    for daemon in &daemons {
        let checks = checks_with_print(daemon)?;

        let worst = checks.iter().map(|check| check.severity).max().unwrap_or(Severity::Ok);
        failed_checks += checks.iter().filter(|check| check.severity == Severity::Fail).count();
        match worst {
            Severity::Fail => failing += 1,
            Severity::Warn => warning += 1,
            Severity::Ok => {}
        }
        if worst == Severity::Ok && !verbose && sweep {
            continue;
        }

        stdout::writeln(&format!(
            "{} ({})",
            styles::prefix(Color::Blue, &daemon.name),
            daemon.domain
        ));
        for check in checks.iter().filter(|check| verbose || check.is_problem()) {
            stdout::writeln(&format!("  {}", styles::check(check)));
        }
        if worst == Severity::Ok && !verbose {
            stdout::writeln(&format!("  {}", styles::check(&Check::ok("no problems found"))));
        }
    }

    if sweep {
        stdout::writeln(&styles::command(&format!(
            "{} services checked: {failing} failing, {warning} with warnings{}",
            daemons.len(),
            if verbose { "" } else { " (-v shows passing checks)" },
        )));
    }
    if failed_checks > 0 { Err(Error::ChecksFailed { failed: failed_checks }) } else { Ok(()) }
}

/// [`doctor::check`] with the loaded definition, so plist edits since the last load are caught.
fn checks_with_print(daemon: &Daemon) -> Result<Vec<Check>> {
    let output = super::print_if_loaded(daemon)?;
    let parsed =
        output.as_ref().and_then(|output| parser::parse_launchctl_service(output, daemon).ok());
    Ok(doctor::check(daemon, parsed.as_ref()))
}
