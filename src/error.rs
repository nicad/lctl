use error_set::error_set;

error_set! {
    /// Plain text error payload
    #[display("{msg}")]
    struct BasicError {
        pub msg: String,
    }

    Error := {
        #[display("Daemon not found: {0}")]
        NotFound(BasicError),

        #[display("Sudo is required to perform this action")]
        NeedsRoot,

        #[display("Invalid argument: {0}")]
        InvalidInput(BasicError),

        #[display("Config error: {0}")]
        Config(BasicError),

        #[display("failed to {action}: {detail}")]
        Launchctl {
            action: String,
            detail: String,
        },

        #[display("{name} is disabled: run `lctl enable {name}` first")]
        Disabled {
            name: String,
        },

        #[display("{failed} doctor check(s) failed")]
        ChecksFailed {
            failed: usize,
        },

        #[display("{0}")]
        Unexpected(BasicError),
    }
}

/// Build a [`BasicError`] from a format string (like `format!`) or any [`Display`] value.
macro_rules! err_msg {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::error::BasicError {
            msg: format!($fmt $(, $arg)*),
        }
    };
    ($m:expr) => {
        $crate::error::BasicError { msg: ($m).to_string() }
    };
}

pub(crate) use err_msg;

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! from_unexpected {
    ($($t:ty),+ $(,)?) => {
        $(
            impl From<$t> for Error {
                fn from(e: $t) -> Self {
                    Error::Unexpected(err_msg!(e))
                }
            }
        )+
    };
}

from_unexpected!(std::io::Error, plist::Error, nix::Error, dialoguer::Error);

impl Error {
    pub fn launchctl(action: impl Into<String>, detail: impl Into<String>) -> Self {
        Error::Launchctl { action: action.into(), detail: detail.into() }
    }

    /// Append a suggestion to a launchctl failure; other errors are returned unchanged.
    pub fn with_hint(self, suggestion: Option<String>) -> Self {
        match (self, suggestion) {
            (Error::Launchctl { action, detail }, Some(suggestion)) => Error::Launchctl {
                action,
                detail: format!("{}\n  hint: {suggestion}", detail.trim_end()),
            },
            (error, _) => error,
        }
    }

    pub fn unexpected(msg: impl std::fmt::Display) -> Self {
        Error::Unexpected(err_msg!(msg))
    }
}
