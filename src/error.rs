use std::fmt;

/// Stable bounded categories; never carry input text, values, or authority messages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    InvalidRequest,
    MalformedToken,
    LimitExceeded,
    AllocationFailed,
    Denied,
    AuthorityUnavailable,
    AuthorityContract,
}

/// Whether authority consumption definitely did not occur, did occur, or is unknown.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitState {
    NotCommitted,
    Committed,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestoreError {
    pub code: ErrorCode,
    pub commit: CommitState,
}
impl RestoreError {
    pub const fn new(code: ErrorCode, commit: CommitState) -> Self {
        Self { code, commit }
    }
    pub(crate) const fn before(code: ErrorCode) -> Self {
        Self::new(code, CommitState::NotCommitted)
    }
    pub(crate) const fn after(code: ErrorCode) -> Self {
        Self::new(code, CommitState::Committed)
    }
}
impl fmt::Display for RestoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "restore {:?} ({:?})", self.code, self.commit)
    }
}
impl std::error::Error for RestoreError {}
