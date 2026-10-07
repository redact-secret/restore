use redact_secret_restore::{
    CommitState, ResolvedValues, RestoreAuthority, RestoreError, RestorePlan,
};

pub const TOKEN: &str = "<rsv_aaaaaaaaaaaaaaaaaaaaaaaaaa>";
/// Benchmark/property-only authority; no policy or lifecycle support is claimed.
pub struct SyntheticAuthority;
impl RestoreAuthority for SyntheticAuthority {
    type Grant = usize;
    fn preflight(&self, plan: &RestorePlan<'_>) -> Result<usize, RestoreError> {
        if plan.occurrences().iter().any(|o| o.token() != TOKEN) {
            return Err(RestoreError::new(
                redact_secret_restore::ErrorCode::Denied,
                CommitState::NotCommitted,
            ));
        }
        Ok(plan.occurrences().len())
    }
    fn consume(
        &mut self,
        count: usize,
        plan: &RestorePlan<'_>,
    ) -> Result<ResolvedValues, RestoreError> {
        if count != plan.occurrences().len() {
            return Err(RestoreError::new(
                redact_secret_restore::ErrorCode::Denied,
                CommitState::NotCommitted,
            ));
        }
        Ok(ResolvedValues::new(
            (0..count).map(|_| "synthetic-value".to_owned()).collect(),
        ))
    }
}
