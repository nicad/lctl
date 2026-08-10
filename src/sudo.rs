use crate::error::{Error, Result};
use crate::output;
use crate::styles;
use crossterm::style::Color;
use nix::unistd;
use std::io::IsTerminal;
use std::process::{Command, Output, Stdio};

/// Whether stdin is an interactive terminal (safe to prompt for a password).
pub fn stdin_is_tty() -> bool {
    std::io::stdin().is_terminal()
}

pub fn is_root() -> bool {
    unistd::geteuid().is_root()
}

/// Ensure we can run `sudo -n` helpers. Already-root is a no-op.
///
/// Non-root + TTY: interactive `sudo -v` (inherited stdio). Non-root + non-TTY: [`Error::NeedsRoot`].
pub fn ensure_elevated() -> Result<()> {
    if is_root() {
        return Ok(());
    }
    if !stdin_is_tty() {
        return Err(Error::NeedsRoot);
    }

    if output::is_verbose() {
        output::stdout::writelogln(styles::prefix(Color::Blue, "CMD"), styles::command("sudo -v"));
    }

    let status = Command::new("sudo")
        .arg("-v")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| Error::unexpected(format!("failed to run sudo -v: {e}")))?;

    if status.success() { Ok(()) } else { Err(Error::NeedsRoot) }
}

/// `sudo -n <program> …` for non-interactive use after [`ensure_elevated`].
pub fn sudo_n(program: &str) -> Command {
    let mut cmd = Command::new("sudo");
    cmd.arg("-n").arg(program);
    cmd
}

/// Run `sudo -n <args…>` and return process output.
pub fn run_sudo_n(args: &[&str]) -> Result<Output> {
    if args.is_empty() {
        return Err(Error::unexpected("sudo -n requires a program"));
    }
    if output::is_verbose() {
        let cmd_str = format!("sudo -n {}", args.join(" "));
        output::stdout::writelogln(styles::prefix(Color::Blue, "CMD"), styles::command(&cmd_str));
    }
    let output = sudo_n(args[0]).args(&args[1..]).output()?;
    Ok(output)
}

/// Run `sudo -n <args…>`; fail with [`Error::NeedsRoot`] or a detail message on non-zero exit.
pub fn require_sudo_n(args: &[&str], action: &str) -> Result<()> {
    let output = run_sudo_n(args)?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("a password is required") || stderr.contains("a terminal is required") {
        return Err(Error::NeedsRoot);
    }
    let detail = if stderr.trim().is_empty() {
        format!("exit code {:?}", output.status.code())
    } else {
        stderr.trim().to_string()
    };
    Err(Error::unexpected(format!("failed to {action}: {detail}")))
}

/// True when launchctl output indicates the op needs root (narrow match).
///
/// Does **not** treat bare `Input/output error` as need-root (user-domain session hint).
pub fn looks_like_needs_root(stdout: &str, stderr: &str) -> bool {
    let combined = if stderr.is_empty() {
        stdout.to_string()
    } else if stdout.is_empty() {
        stderr.to_string()
    } else {
        format!("{stdout}\n{stderr}")
    };

    if combined.contains("Try re-running the command as root") {
        return true;
    }

    // SIP denial is not fixed by sudo.
    if combined.contains("System Integrity Protection") {
        return false;
    }

    combined.contains("Operation not permitted")
}

/// Convenience for unit tests / callers with owned strings.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_root_hint_line() {
        assert!(looks_like_needs_root(
            "",
            "Bootstrap failed: 5: Input/output error\nTry re-running the command as root for richer errors.\n"
        ));
    }

    #[test]
    fn detects_operation_not_permitted() {
        assert!(looks_like_needs_root("", "Boot-out failed: 1: Operation not permitted\n"));
        assert!(looks_like_needs_root(
            "",
            "Could not enable service: 1: Operation not permitted\n"
        ));
    }

    #[test]
    fn ignores_io_error_alone() {
        assert!(!looks_like_needs_root("", "Bootstrap failed: 5: Input/output error\n"));
    }

    #[test]
    fn ignores_sip() {
        assert!(!looks_like_needs_root(
            "",
            "Boot-out failed: 150: Operation not permitted while System Integrity Protection is engaged\n"
        ));
    }

    #[test]
    fn ignores_unrelated() {
        assert!(!looks_like_needs_root("", "Boot-out failed: 3: No such process\n"));
        assert!(!looks_like_needs_root("Bad request.\n", ""));
    }
}
