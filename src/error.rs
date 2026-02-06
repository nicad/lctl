use crate::output::stderr;
use thiserror::Error;

/// User-facing errors - validation, input errors, user mistakes
/// These should be printed to stdout as they're part of normal user interaction
#[derive(Error, Debug)]
pub enum UserError {
    #[error("Daemon not found: {0}")]
    DaemonNotFound(String),

    #[error("Sudo is required to perform this action")]
    RequiresRoot,

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

/// System/internal errors - I/O failures, parsing errors, system call failures
/// These should be printed to stderr as they indicate internal problems
#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Config error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Plist parse error: {0}")]
    Plist(#[from] plist::Error),

    #[error("Nix error: {0}")]
    Nix(#[from] nix::Error),

    #[error("Dialoguer error: {0}")]
    Dialoguer(#[from] dialoguer::Error),

    #[error("Launchctl error: {0}")]
    Launchctl(String),
}

/// Unified error type that can be either a user error or system error
#[derive(Error, Debug)]
pub enum AngelError {
    #[error(transparent)]
    User(#[from] UserError),

    #[error(transparent)]
    System(#[from] SystemError),
}

// Implement From for underlying error types so ? operator works
impl From<std::io::Error> for AngelError {
    fn from(err: std::io::Error) -> Self {
        AngelError::System(SystemError::Io(err))
    }
}

impl From<plist::Error> for AngelError {
    fn from(err: plist::Error) -> Self {
        AngelError::System(SystemError::Plist(err))
    }
}

impl From<nix::Error> for AngelError {
    fn from(err: nix::Error) -> Self {
        AngelError::System(SystemError::Nix(err))
    }
}

impl From<dialoguer::Error> for AngelError {
    fn from(err: dialoguer::Error) -> Self {
        AngelError::System(SystemError::Dialoguer(err))
    }
}

// Enable `?` operator for nom parsing errors
impl From<nom::Err<nom::error::Error<&str>>> for SystemError {
    fn from(e: nom::Err<nom::error::Error<&str>>) -> Self {
        let error_msg = format!("Parse error: {:?}", e);
        stderr::error(&error_msg);
        SystemError::Launchctl(error_msg)
    }
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for SystemError {
    fn from(e: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        let error_msg = format!("Parse error: {:?}", e);
        stderr::error(&error_msg);
        SystemError::Launchctl(error_msg)
    }
}

pub type Result<T> = std::result::Result<T, AngelError>;
