use crate::cli::{ListArgs, SortBy};
use crate::describe;
use crate::error::Result;
use crate::lctl::Lctl;
use crate::output;
use crate::output::stdout;
use crate::process;
use crate::styles;
use crate::types::{Daemon, ForWhom, ServiceState};
use comfy_table::Cell;
use std::time::SystemTime;

pub(crate) fn run(lctl: &Lctl, args: &ListArgs) -> Result<()> {
    let query = args.pattern.as_deref().unwrap_or("");
    let mut daemons: Vec<&Daemon> = lctl
        .daemons
        .get_matches(query, args.exact)
        .into_iter()
        .filter(|daemon| is_listed(daemon, args))
        .collect();
    sort_daemons(args.sort_by, &mut daemons);

    if args.labels_only {
        for daemon in &daemons {
            stdout::writeln(&daemon.name);
        }
        return Ok(());
    }

    let now = SystemTime::now();
    let mut table = styles::create_table();
    let mut header = vec!["State", "PID", "Started", "Up", "Runs when", "Label"];
    if args.long {
        header.extend(["Domain", "Plist"]);
    }
    table.set_header(header);
    for daemon in &daemons {
        let start = daemon.pid.and_then(process::start_time);
        let mut row: Vec<Cell> = vec![
            styles::state_cell(&daemon.state()),
            daemon.pid.map_or("-".to_string(), |pid| pid.to_string()).into(),
            start.map(|start| describe::started(start, now)).unwrap_or_default().into(),
            start.map(|start| describe::uptime(start, now, 1)).unwrap_or_default().into(),
            daemon.plist.as_ref().map_or("-".to_string(), describe::runs_when).into(),
            daemon.name.clone().into(),
        ];
        if args.long {
            row.push(daemon.domain.to_string().into());
            row.push(styles::display_path(daemon, output::is_verbose()).into());
        }
        table.add_row(row);
    }
    stdout::writeln(&table);
    stdout::writeln(&summary(&daemons));
    Ok(())
}

/// What `list` shows by default (with a plist on disk, not Apple's), sorted by label.
pub(crate) fn your_services(lctl: &Lctl) -> Vec<&Daemon> {
    let defaults = ListArgs::default();
    let mut daemons: Vec<&Daemon> = lctl
        .daemons
        .get_matches("", false)
        .into_iter()
        .filter(|daemon| is_listed(daemon, &defaults))
        .collect();
    sort_daemons(SortBy::Name, &mut daemons);
    daemons
}

fn is_listed(daemon: &Daemon, args: &ListArgs) -> bool {
    if (daemon.for_use_by == ForWhom::Apple && !args.show_apple)
        || (daemon.source_path.is_none() && !args.show_dynamic)
    {
        return false;
    }
    if !(args.running || args.idle || args.failed || args.not_loaded || args.disabled) {
        return true;
    }
    match daemon.state() {
        ServiceState::Running => args.running,
        ServiceState::Idle => args.idle,
        ServiceState::Failed(_) => args.failed,
        ServiceState::NotLoaded => args.not_loaded,
        ServiceState::Disabled => args.disabled,
    }
}

/// e.g. "7 services: 4 running, 1 failed, 2 not loaded · lctl status <name> for details"
fn summary(daemons: &[&Daemon]) -> String {
    let breakdown: Vec<String> = ServiceState::NAMES
        .iter()
        .filter_map(|name| {
            let count = daemons.iter().filter(|daemon| daemon.state().name() == *name).count();
            (count > 0).then(|| format!("{count} {name}"))
        })
        .collect();
    let services = if daemons.len() == 1 { "service" } else { "services" };
    styles::command(&format!(
        "{} {services}{}{} · lctl status <name> for details · lctl doctor checks for problems",
        daemons.len(),
        if breakdown.is_empty() { "" } else { ": " },
        breakdown.join(", "),
    ))
}

fn sort_daemons(sort_by: SortBy, daemons: &mut Vec<&Daemon>) {
    match sort_by {
        SortBy::Name => {
            daemons.sort_by(|a, b| a.name.cmp(&b.name));
        }
        SortBy::Domain => {
            daemons.sort_by(|a, b| {
                a.domain.to_string().cmp(&b.domain.to_string()).then_with(|| a.name.cmp(&b.name))
            });
        }
        SortBy::Parent => {
            daemons.sort_by(|a, b| {
                get_parent_path(a).cmp(get_parent_path(b)).then_with(|| a.name.cmp(&b.name))
            });
        }
    }
}

fn get_parent_path(daemon: &Daemon) -> &str {
    daemon
        .source_path
        .as_ref()
        .and_then(|p| p.parent())
        .and_then(|p| p.to_str())
        .unwrap_or_default()
}
