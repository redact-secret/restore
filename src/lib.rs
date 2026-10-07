//! Bounded synchronous controlled reconstruction. Syntax is never authorization.
//! The host supplies trusted context; the authority owns policy and consumption.
//! No dependencies, persistence, token issuance, async, or streaming API.
#![forbid(unsafe_code)]
mod authority;
mod error;
mod limits;
mod plan;
mod reconstruct;
mod request;
mod scanner;

pub use authority::{ResolvedValues, RestoreAuthority};
pub use error::{CommitState, ErrorCode, RestoreError};
pub use limits::Limits;
pub use plan::{Occurrence, RestorePlan};
pub use reconstruct::RestoreResult;
pub use request::{RestoreField, RestoreRequest, TrustedContext};

/// Validate/scan/plan, bulk preflight, atomic consume, then reconstruct once.
/// A post-consume error releases no output but may burn token uses.
pub fn restore<A: RestoreAuthority>(
    request: &RestoreRequest<'_>,
    authority: &mut A,
    limits: Limits,
) -> Result<RestoreResult, RestoreError> {
    let plan = RestorePlan::build(request, limits)?;
    restore_plan(&plan, authority)
}

/// Execute an immutable prebuilt plan without rescanning. Eligibility is checked
/// afresh on every call; retaining a plan does not reserve authority or budgets.
pub fn restore_plan<A: RestoreAuthority>(
    plan: &RestorePlan<'_>,
    authority: &mut A,
) -> Result<RestoreResult, RestoreError> {
    let grant = authority
        .preflight(plan)
        .map_err(|error| RestoreError::new(error.code, CommitState::NotCommitted))?;
    let values = authority.consume(grant, plan)?;
    reconstruct::reconstruct(plan, values)
}
