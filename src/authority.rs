use crate::{RestoreError, RestorePlan};
use std::fmt;

/// Complete owned values, exactly one per plan occurrence in occurrence order.
/// The authority is trusted to associate values with this exact plan. No general
/// mapping iterator or per-token resolution API is exposed by the engine.
pub struct ResolvedValues(pub(crate) Vec<String>);
impl ResolvedValues {
    /// Construct a complete handoff after atomic consumption. The engine checks
    /// length and output bounds; authorization/value association belongs to you.
    pub fn new(values: Vec<String>) -> Self {
        Self(values)
    }
}
impl fmt::Debug for ResolvedValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolvedValues")
            .field("count", &self.0.len())
            .finish_non_exhaustive()
    }
}

/// Synchronous, bulk, request-bound authority contract v0.1.
///
/// Preflight returns no plaintext and consumes no budgets. A grant is not a
/// reservation. Consume must bind the grant to this plan and revalidate current
/// policy/expiry/revocation/budgets at its atomic linearization point. Duplicates
/// consume one use per occurrence. A denied batch consumes no unrelated uses.
///
/// On consume failure return a bounded error with truthful NotCommitted,
/// Committed, or Indeterminate state. Unknown outcomes release no values and
/// must not be blindly retried. On success consumption is committed; subsequent
/// engine failure cannot refund it. Never log values or propagate callback text.
/// See docs/authority-semantics.md. No async/cancellation guarantee is implied.
pub trait RestoreAuthority {
    type Grant;
    fn preflight(&self, plan: &RestorePlan<'_>) -> Result<Self::Grant, RestoreError>;
    fn consume(
        &mut self,
        grant: Self::Grant,
        plan: &RestorePlan<'_>,
    ) -> Result<ResolvedValues, RestoreError>;
}
