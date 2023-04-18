#[derive(Debug, thiserror::Error, Clone)]
pub enum ConnectionError {
    #[error("Timed out")]
    TimedOut,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("{msg:?}")]
    Generic { msg: String },
}

impl<'t, 's> From<libstrophe::ConnectionError<'t, 's>> for ConnectionError {
    fn from(error: libstrophe::ConnectionError<'t, 's>) -> Self {
        match error {
            libstrophe::ConnectionError::Aborted => ConnectionError::Generic {
                msg: error.to_string(),
            },
            libstrophe::ConnectionError::TimedOut => ConnectionError::TimedOut,
            libstrophe::ConnectionError::ConnectionReset => ConnectionError::Generic {
                msg: error.to_string(),
            },
            libstrophe::ConnectionError::TLS(_) => ConnectionError::Generic {
                msg: error.to_string(),
            },
            libstrophe::ConnectionError::Stream(_) => ConnectionError::Generic {
                msg: error.to_string(),
            },
        }
    }
}
