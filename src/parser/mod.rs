pub mod domain;
pub mod service;

pub use service::{ParsedService, ServiceItem};

pub struct Parser;

impl Parser {
    pub fn parse_print_domain(
        domain: &crate::types::Domain,
    ) -> crate::error::Result<Vec<(Option<u32>, Option<String>, String)>> {
        domain::parse_print_domain(domain)
    }

    pub fn parse_print_service<'a>(
        result: &'a crate::launchctl::LaunchctlOutput,
        daemon: &crate::types::Daemon,
    ) -> crate::error::Result<ParsedService<'a>> {
        let output = result.require_print_output(&daemon.name)?;
        service::parse_print_service(output)
    }
}
