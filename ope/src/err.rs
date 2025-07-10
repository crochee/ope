use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(
        "The request was denied because a statement denied request.Please proofread the policy {0}"
    )]
    Deny(String),
    #[error("The request was denied because no matching statement was found.")]
    NotMatched,
    #[error("invalid cache size {0}")]
    InvalidCacheSize(usize),
    #[error("lock error: {0}")]
    LockError(String),
    #[error(transparent)]
    CompileRegexError(#[from] regex::Error),
    #[error("Unbalanced braces in {0}")]
    UnbalancedBraces(String),
    #[error("{0}")]
    NotIndex(String),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error("Could not find condition type {0}")]
    NotFoundConditionType(String),
}
