use crate::error::{Error, Result};
use crate::launchctl;
use crate::types::Domain;
use nom::{
    AsChar, IResult, Input, Parser,
    bytes::complete::{tag, take_until1},
    character::complete::{multispace0, multispace1},
    error::ErrorKind,
    multi::many_till,
    sequence::{preceded, terminated},
};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainService {
    pub pid: Option<u32>,
    pub last_exit_code: Option<String>,
    pub name: String,
}

pub struct DomainListing {
    pub services: Vec<DomainService>,
    /// Labels with launchd's disabled flag set in this domain (loaded or not).
    pub disabled: HashSet<String>,
}

pub fn parse_print_domain(domain: &Domain) -> Result<DomainListing> {
    let result = launchctl::print(&domain.to_string())?;
    result.require_ok(&format!("print domain {}", domain))?;
    Ok(DomainListing {
        services: parse_domain_output(result.stdout.as_bytes())?,
        disabled: parse_disabled_services(&result.stdout),
    })
}

/// Labels marked `=> disabled` in the `disabled services = { … }` block.
pub fn parse_disabled_services(input: &str) -> HashSet<String> {
    let Some(start) = input.find("disabled services = {") else {
        return HashSet::new();
    };
    input[start..]
        .lines()
        .skip(1)
        .take_while(|line| line.trim() != "}")
        .filter_map(|line| {
            let (label, state) = line.trim().rsplit_once(" => ")?;
            (state == "disabled").then(|| label.trim_matches('"').to_string())
        })
        .collect()
}

pub fn parse_domain_output(input: &[u8]) -> Result<Vec<DomainService>> {
    let services_marker: &[u8] = b"services = {";
    let closing_brace_parser = preceded(multispace0, tag(b"}" as &[u8]));

    let mut parser = (
        terminated(take_until1(services_marker), tag(services_marker)),
        many_till(parse_service_line, closing_brace_parser),
    );
    let (_, (_, (services, _))) = parser.parse(input).map_err(|e| Error::unexpected(e))?;
    Ok(services)
}

fn parse_service_line(input: &[u8]) -> IResult<&[u8], DomainService> {
    let (input, _) = multispace0(input)?;
    let (input, pid_bytes) = terminated(non_whitespace1, multispace1).parse(input)?;
    let (input, exit_code_bytes) = terminated(non_whitespace1, multispace1).parse(input)?;
    let (input, label_bytes) = terminated(non_whitespace1, multispace0).parse(input)?;

    // Not-running services are listed with pid `-` or `0`.
    let pid = std::str::from_utf8(pid_bytes)
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|&pid| pid != 0);
    let last_exit_code = std::str::from_utf8(exit_code_bytes).ok().map(|s| s.to_string());
    let name = String::from_utf8_lossy(label_bytes).trim().to_string();

    Ok((input, DomainService { pid, last_exit_code, name }))
}

fn non_whitespace1(input: &[u8]) -> IResult<&[u8], &[u8]> {
    input.split_at_position1_complete(|item| item.as_char().is_whitespace(), ErrorKind::Char)
}
