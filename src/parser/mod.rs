pub mod domain;
pub mod service;

pub use domain::{DomainService, parse_domain_output, parse_print_domain};
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
