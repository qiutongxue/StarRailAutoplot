use thiserror::Error;

#[derive(Debug, Error)]
pub enum XCapError {
    #[error("{0}")]
    Error(String),
    #[cfg(target_os = "windows")]
    #[error(transparent)]
    WindowsCoreError(#[from] windows::core::Error),
    #[cfg(target_os = "windows")]
    #[error(transparent)]
    StdStringFromUtf16Error(#[from] std::string::FromUtf16Error),
}

impl XCapError {
    pub fn new<S: ToString>(err: S) -> Self {
        XCapError::Error(err.to_string())
    }
}

pub type XCapResult<T> = Result<T, XCapError>;
