use thiserror::Error;

pub type Result<T> = std::result::Result<T, AngelError>;

#[derive(Error, Debug)]
pub enum AngelError {
    /// USER ERRORS

    #[error("Daemon not found: {0}")]
    DaemonNotFound(String),

    #[error("Sudo is required to perform this action")]
    RequiresRoot,

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Config error: {0}")]
    Config(String),

    /// INTERNAL ERRORS

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

    #[error("Parser error: {0}")]
    Parser(String),
}

impl AngelError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AngelError::DaemonNotFound(_) => 1,
            AngelError::RequiresRoot => 77,
            AngelError::InvalidArgument(_) => 64,
            AngelError::Config(_) => 78,
            _ => 1,
        }
    }
}
