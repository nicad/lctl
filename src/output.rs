use crossterm::style::Color;
use std::io::{self, Write};
use std::sync::OnceLock;

/// Global output context storing configuration
pub struct OutputContext {
    pub verbose: bool,
}

static CONTEXT: OnceLock<OutputContext> = OnceLock::new();

/// Initialize the output context with the verbose flag
/// Must be called before any output functions are used
pub fn init(verbose: bool) {
    CONTEXT.set(OutputContext { verbose }).ok();
}

#[inline(always)]
fn context() -> &'static OutputContext {
    CONTEXT.get_or_init(|| OutputContext { verbose: false })
}

/// Check if verbose output is enabled
#[inline(always)]
pub fn is_verbose() -> bool {
    context().verbose
}

macro_rules! write_to_stream {
    (io::$stream:ident, $msg:expr, $newline:expr) => {
        let result = match $newline {
            true => writeln!(io::$stream(), "{}", $msg),
            false => write!(io::$stream(), "{}", $msg),
        };
        if let Err(e) = result {
            let _ = writeln!(io::stderr(), "Failed to write to {}: {}", stringify!($stream), e);
        }
    };
}

/// User-facing data output (stdout) - for structured data, tables, results
pub mod stdout {
    use super::*;
    use crate::styles::styles::prefix;

    #[inline(always)]
    pub fn write(data: impl std::fmt::Display) {
        write_to_stream!(io::stdout, data, false);
    }

    #[inline(always)]
    pub fn writeln(msg: impl std::fmt::Display) {
        write_to_stream!(io::stdout, msg, true);
    }

    #[inline(always)]
    pub fn writelogln(prefix: impl std::fmt::Display, msg: impl std::fmt::Display) {
        write_to_stream!(io::stdout, format!("{} {}", prefix, msg), true);
    }

    #[inline(always)]
    pub fn success(msg: &str) {
        writelogln(prefix(Color::Green, "✔"), msg);
    }

    #[inline(always)]
    pub fn hint(msg: &str) {
        writelogln(prefix(Color::Yellow, "→"), msg);
    }
}

/// Error/log output (stderr) - for errors, warnings, debug info
pub mod stderr {
    use crate::styles::styles::prefix;
    use crossterm::style::Color;
    use std::io::{self, Write};

    #[inline(always)]
    pub fn writelogln(prefix: impl std::fmt::Display, msg: impl std::fmt::Display) {
        write_to_stream!(io::stderr, format!("{:<7} {}", prefix, msg), true);
    }

    #[inline(always)]
    pub fn warn(msg: &str) {
        writelogln(prefix(Color::Yellow, "WARN"), msg);
    }

    #[inline(always)]
    pub fn error(msg: &str) {
        writelogln(prefix(Color::Red, "ERROR"), msg);
    }
}
