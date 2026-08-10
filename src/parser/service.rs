use crate::error::{Error as AngelError, Result};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{is_not, tag, take_until, take_until1},
    character::complete::{char, newline, tab},
    combinator::opt,
    error::{Error, ErrorKind},
    multi::{many0, many0_count},
    sequence::{delimited, terminated},
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceItem<S = String> {
    Str(S),
    Array(Vec<ServiceItem<S>>),
    Dict(Vec<ServiceItem<S>>),
    KeyValue(S, Box<ServiceItem<S>>),
}

/// Parsed service output: top-level key → value (first occurrence per key).
pub type ParsedService<'a> = HashMap<&'a str, ServiceItem<&'a str>>;

/// Parses launchctl output
/// Takes the validated output string (after LaunchctlOutput::require_print_output)
pub fn parse_print_service<'a>(output: &'a str) -> Result<ParsedService<'a>> {
    let (_, lines) =
        many0(parse_line).parse(output).map_err(|e| AngelError::unexpected(e))?;
    let items = match lines.first() {
        Some(ServiceItem::KeyValue(_, boxed)) => match boxed.as_ref() {
            ServiceItem::Dict(inner) => inner,
            _ => {
                return Err(AngelError::unexpected("expected root dict"));
            }
        },
        _ => {
            return Err(AngelError::unexpected("expected root dict"));
        }
    };
    let map: HashMap<_, _> = items
        .clone()
        .into_iter()
        .filter_map(|item| match item {
            ServiceItem::KeyValue(k, v) => Some((k, *v)),
            _ => None,
        })
        .collect();
    Ok(map)
}

/// Returns one line as ServiceItem. KeyValue(key, value) for dict entries; bare value for array elements.
fn parse_line<'a>(input: &'a str) -> IResult<&'a str, ServiceItem<&'a str>> {
    if input.is_empty() {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
    }

    let (input, _) = many0(newline).parse(input)?; // skip preceding newlines
    let (input, indent) = many0_count(tab).parse(input)?; // count indent level
    let (remaining, line) = is_not("\n").parse(input)?; // get content up to newline

    let (value_str, key) = parse_key_value.parse(line)?;
    let (rest, value) = match value_str {
        // parse nested section
        "{" => {
            let (rest_after_brace, section) = get_section_until_closing_brace(remaining, indent)?;
            let (_, items) = many0(parse_line).parse(section)?;
            let value = match items.first() {
                // decide dict or array
                Some(ServiceItem::KeyValue(_, _)) => ServiceItem::Dict(items),
                _ => ServiceItem::Array(items),
            };
            (rest_after_brace, value)
        }
        _ => (remaining, ServiceItem::Str(value_str)),
    };

    let item = match key {
        None => value,
        Some(k) => ServiceItem::KeyValue(k, Box::new(value)),
    };
    Ok((rest, item))
}

/// Returns key as Some(key) or None (array item); value is remaining input.
fn parse_key_value<'a>(input: &'a str) -> IResult<&'a str, Option<&'a str>> {
    opt(terminated(
        alt((delimited(char('"'), take_until1("\""), char('"')), take_until1(" ="))),
        delimited(tag(" "), alt((tag("=>"), tag("="))), tag(" ")),
    ))
    .parse(input)
}

/// Finds the closing brace at the expected indent level and returns the section up to (but not including) that line.
fn get_section_until_closing_brace(input: &str, expected_indent: usize) -> IResult<&str, &str> {
    let close_pattern = format!("\n{}}}", "\t".repeat(expected_indent));
    let (remaining, section) = take_until(close_pattern.as_str()).parse(input)?;
    let (remaining, _) = tag(close_pattern.as_str()).parse(remaining)?;
    Ok((remaining, section))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flat_service_fields() {
        let output = "\
com.example.svc = {
\tstate = running
\tpid = 4242
\tpath = /tmp/com.example.svc.plist
}
";
        let parsed = parse_print_service(output).unwrap();
        assert_eq!(parsed.get("state"), Some(&ServiceItem::Str("running")));
        assert_eq!(parsed.get("pid"), Some(&ServiceItem::Str("4242")));
        assert_eq!(
            parsed.get("path"),
            Some(&ServiceItem::Str("/tmp/com.example.svc.plist"))
        );
    }

    #[test]
    fn parses_nested_array_and_dict() {
        let output = "\
com.example.svc = {
\targuments = {
\t\t/usr/bin/foo
\t\t--flag
\t}
\tenvironment = {
\t\t\"PATH\" => /usr/bin
\t}
}
";
        let parsed = parse_print_service(output).unwrap();
        match parsed.get("arguments") {
            Some(ServiceItem::Array(items)) => {
                assert_eq!(items, &vec![ServiceItem::Str("/usr/bin/foo"), ServiceItem::Str("--flag")]);
            }
            other => panic!("expected arguments array, got {other:?}"),
        }
        match parsed.get("environment") {
            Some(ServiceItem::Dict(items)) => {
                assert_eq!(
                    items,
                    &vec![ServiceItem::KeyValue(
                        "PATH",
                        Box::new(ServiceItem::Str("/usr/bin"))
                    )]
                );
            }
            other => panic!("expected environment dict, got {other:?}"),
        }
    }

    #[test]
    fn rejects_non_dict_root() {
        assert!(parse_print_service("not a dict\n").is_err());
    }
}

