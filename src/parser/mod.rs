pub mod domain;
pub mod service;

pub use domain::{
    DomainListing, DomainService, parse_disabled_services, parse_domain_output, parse_print_domain,
};
pub use service::{ParsedService, ServiceItem, parse_print_service};

use crate::error::Result;
use crate::launchctl::LaunchctlOutput;
use crate::types::Daemon;

/// Validate launchctl print output then parse the service dict.
pub fn parse_launchctl_service<'a>(
    result: &'a LaunchctlOutput,
    daemon: &Daemon,
) -> Result<ParsedService<'a>> {
    let output = result.require_print_output(&daemon.name)?;
    parse_print_service(output)
}

/// A string field of a parsed `launchctl print`.
pub fn str_field<'a>(parsed: &ParsedService<'a>, key: &str) -> Option<&'a str> {
    match parsed.get(key) {
        Some(ServiceItem::Str(value)) => Some(value),
        _ => None,
    }
}

/// A list field (e.g. `arguments = { … }`) of a parsed `launchctl print`.
pub fn list_field<'a>(parsed: &ParsedService<'a>, key: &str) -> Option<Vec<&'a str>> {
    match parsed.get(key) {
        Some(ServiceItem::Array(items)) => Some(
            items
                .iter()
                .filter_map(|item| match item {
                    ServiceItem::Str(value) => Some(*value),
                    _ => None,
                })
                .collect(),
        ),
        _ => None,
    }
}
