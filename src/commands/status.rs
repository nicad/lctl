use crate::angel::Angel;
use crate::cli::StatusArgs;
use crate::error::Result;
use crate::launchctl;
use crate::output::stdout;
use crate::parser::{Parser, ServiceItem};
use crate::styles::styles;
use comfy_table::Table;
use crossterm::style::Color;

pub fn run(angel: &Angel, args: &StatusArgs) -> Result<()> {
    let daemon = angel.daemons.get_match(&args.name, args.exact)?;

    // Get launchctl result once (reuse for debug and parsing)
    let result = launchctl::print(daemon)?;

    // Debug output
    if args.debug {
        stdout::writeln(&styles::prefix(Color::Yellow, "=== Rendered launchctl print output ==="));
        stdout::writeln(&result.output);
        stdout::writeln("");
        stdout::writeln(&styles::prefix(Color::Yellow, "=== Raw launchctl print output ==="));
        let escaped_output = result.output.replace('\n', "\\n").replace('\t', "\\t");
        stdout::writeln(&escaped_output);
        stdout::writeln("");
    }

    let parsed = Parser::parse_print_service(&result, daemon)?;

    if args.debug {
        stdout::writeln(&styles::prefix(Color::Yellow, "=== Parsed output (pretty print) ==="));
        stdout::writeln(&format!("{:#?}", parsed));
        stdout::writeln("");
    }

    let status = parsed
        .get("state")
        .and_then(|v| match v {
            ServiceItem::Str(s) => Some(*s),
            _ => None,
        })
        .unwrap_or("unknown");
    let color = if status == "running" { None } else { Some(Color::Red) };

    stdout::writeln(&styles::prefix(Color::Blue, &daemon.name));
    stdout::writeln(&styles::format_status_dot(status, color));

    let mut table = styles::create_table();
    let mut shown_fields = std::collections::HashSet::new();

    add_field_if_present(&mut table, &parsed, &mut shown_fields, "state", "State");
    add_field_if_present(&mut table, &parsed, &mut shown_fields, "pid", "PID");
    add_field_if_present(&mut table, &parsed, &mut shown_fields, "path", "Path");
    add_field_if_present(&mut table, &parsed, &mut shown_fields, "program", "Program");

    if daemon.source_path.is_some() {
        table.add_row(vec!["Source".to_string(), styles::display_path(daemon, true)]);
    }

    format_arguments(&mut table, &parsed, &mut shown_fields);

    let mut remaining: Vec<_> = parsed.iter().map(|(k, v)| (*k, v)).collect();
    remaining.sort_by_key(|(k, _)| *k);

    for (key, value) in remaining {
        if shown_fields.contains(key) {
            continue;
        }
        match value {
            ServiceItem::Dict(items) => {
                table.add_row(vec![key.to_string(), create_nested_table(items)]);
            }
            ServiceItem::Array(_) | ServiceItem::Str(_) | ServiceItem::KeyValue(_, _) => {
                let display_value = format_value_for_display(value);
                table.add_row(vec![key.to_string(), display_value]);
            }
        }
    }

    stdout::writeln(&table);
    Ok(())
}

fn add_field_if_present<'a>(
    table: &mut Table,
    parsed: &crate::parser::ParsedService<'_>,
    shown_fields: &mut std::collections::HashSet<&'a str>,
    key: &'a str,
    label: &str,
) {
    if let Some(ServiceItem::Str(s)) = parsed.get(key) {
        table.add_row(vec![label.to_string(), (*s).to_string()]);
        shown_fields.insert(key);
    }
}

fn format_arguments(
    table: &mut Table,
    parsed: &crate::parser::ParsedService<'_>,
    shown_fields: &mut std::collections::HashSet<&str>,
) {
    if let Some(ServiceItem::Array(args)) = parsed.get("arguments") {
        if !args.is_empty() {
            let arg_strings: Vec<String> = args
                .iter()
                .filter_map(|v| match v {
                    ServiceItem::Str(s) => Some((*s).to_string()),
                    _ => None,
                })
                .collect();
            if !arg_strings.is_empty() {
                table.add_row(vec!["Arguments".to_string(), arg_strings.join(" ")]);
            }
        }
        shown_fields.insert("arguments");
    }
}

fn format_value_for_display(value: &ServiceItem<&str>) -> String {
    match value {
        ServiceItem::Str(s) => (*s).to_string(),
        ServiceItem::Array(items) => {
            let formatted: Vec<String> = items.iter().map(format_value_for_display).collect();
            format!("[{}]", formatted.join(", "))
        }
        ServiceItem::Dict(_) => unreachable!("Dicts are handled by create_nested_table()"),
        ServiceItem::KeyValue(_, v) => format_value_for_display(v),
    }
}

fn create_nested_table(items: &[ServiceItem<&str>]) -> String {
    let pairs: Vec<(&str, &ServiceItem<&str>)> = items
        .iter()
        .filter_map(|item| match item {
            ServiceItem::KeyValue(k, v) => Some((*k, v.as_ref())),
            _ => None,
        })
        .collect();
    if pairs.is_empty() {
        return "{}".to_string();
    }

    let mut nested_table = styles::create_table();
    let mut sorted: Vec<_> = pairs.into_iter().collect();
    sorted.sort_by_key(|(k, _)| *k);

    for (key, value) in sorted {
        match value {
            ServiceItem::Dict(nested_items) => {
                let nested_table_str = create_nested_table(nested_items);
                nested_table.add_row(vec![key.to_string(), nested_table_str]);
            }
            _ => {
                let display_value = format_value_for_display(value);
                nested_table.add_row(vec![key.to_string(), display_value]);
            }
        }
    }

    nested_table.to_string()
}
