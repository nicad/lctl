use crate::cli::{ListArgs, SortBy};
use crate::error::Result;
use crate::lctl::Lctl;
use crate::output;
use crate::output::stdout;
use crate::styles;
use crate::types::ForWhom;

pub(crate) fn run(lctl: &Lctl, args: &ListArgs) -> Result<()> {
    let query = args.pattern.as_deref().unwrap_or("");
    let mut matching_daemons = lctl.daemons.get_matches(query, args.exact);
    sort_daemons(args.sort_by, &mut matching_daemons);

    let mut table = styles::create_table();
    table.set_header(vec!["EC", "PID", "Domain", "Name", "Source"]);

    for daemon in &matching_daemons {
        if (daemon.for_use_by == ForWhom::Apple && !args.show_apple)
            || (daemon.source_path.is_none() && !args.show_dynamic)
            || (daemon.pid.is_none() && !args.show_idle)
        {
            continue;
        }

        table.add_row(vec![
            daemon.last_exit_code.clone().unwrap_or("-".to_string()),
            daemon.pid.map_or("-".to_string(), |p| p.to_string()),
            daemon.domain.to_string(),
            daemon.name.clone(),
            styles::display_path(daemon, output::is_verbose()),
        ]);
    }

    stdout::writeln(&table);
    Ok(())
}

fn sort_daemons(sort_by: SortBy, daemons: &mut Vec<&crate::types::Daemon>) {
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

fn get_parent_path(daemon: &crate::types::Daemon) -> &str {
    daemon
        .source_path
        .as_ref()
        .and_then(|p| p.parent())
        .and_then(|p| p.to_str())
        .unwrap_or_default()
}
