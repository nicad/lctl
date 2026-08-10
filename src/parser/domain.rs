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

pub fn parse_print_domain(domain: &Domain) -> Result<Vec<(Option<u32>, Option<String>, String)>> {
    let result = launchctl::print(&domain.to_string())?;
    result.require_ok(&format!("print domain {}", domain))?;
    parse_domain_output(result.stdout.as_bytes())
}

fn parse_domain_output(input: &[u8]) -> Result<Vec<(Option<u32>, Option<String>, String)>> {
    let services_marker: &[u8] = b"services = {";
    let closing_brace_parser = preceded(multispace0, tag(b"}" as &[u8]));

    let mut parser = (
        terminated(take_until1(services_marker), tag(services_marker)),
        many_till(parse_service_line, closing_brace_parser),
    );
    let (_, (_, (services, _))) =
        parser.parse(input).map_err(|e| Error::unexpected(e))?;
    Ok(services)
}

fn parse_service_line(input: &[u8]) -> IResult<&[u8], (Option<u32>, Option<String>, String)> {
    let (input, _) = multispace0(input)?;
    let (input, pid_bytes) = terminated(non_whitespace1, multispace1).parse(input)?;
    let (input, exit_code_bytes) = terminated(non_whitespace1, multispace1).parse(input)?;
    let (input, label_bytes) = terminated(non_whitespace1, multispace0).parse(input)?;

    let pid = std::str::from_utf8(pid_bytes).ok().and_then(|s| s.parse::<u32>().ok());
    let exit_code = std::str::from_utf8(exit_code_bytes).ok().map(|s| s.to_string());
    let label = String::from_utf8_lossy(label_bytes).trim().to_string();

    Ok((input, (pid, exit_code, label)))
}

fn non_whitespace1(input: &[u8]) -> IResult<&[u8], &[u8]> {
    input.split_at_position1_complete(|item| item.as_char().is_whitespace(), ErrorKind::Char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_services_block() {
        let input = br#"gui/501 = {
	type = prompt
	services = {
		123	0	com.example.running
		-	78	com.example.idle
	}
	disabled services = {
	}
}
"#;
        let services = parse_domain_output(input).unwrap();
        assert_eq!(services.len(), 2);
        assert_eq!(
            services[0],
            (Some(123), Some("0".into()), "com.example.running".into())
        );
        assert_eq!(
            services[1],
            (None, Some("78".into()), "com.example.idle".into())
        );
    }

    #[test]
    fn rejects_missing_services_block() {
        let input = b"gui/501 = {\n\ttype = prompt\n}\n";
        assert!(parse_domain_output(input).is_err());
    }
}

