use crate::{ErrorCode, ResolvedValues, RestoreError, RestorePlan};
use std::fmt;

/// Complete request result. Debug deliberately omits plaintext and paths.
pub struct RestoreResult {
    fields: Vec<String>,
    restored: usize,
}
impl RestoreResult {
    /// Fields in request order; host must route them to the authorized sink/paths.
    pub fn fields(&self) -> &[String] {
        &self.fields
    }
    pub fn into_fields(self) -> Vec<String> {
        self.fields
    }
    pub fn restored(&self) -> usize {
        self.restored
    }
}
impl fmt::Debug for RestoreResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RestoreResult")
            .field("fields", &self.fields.len())
            .field("restored", &self.restored)
            .finish_non_exhaustive()
    }
}
pub(crate) fn reconstruct(
    plan: &RestorePlan<'_>,
    values: ResolvedValues,
) -> Result<RestoreResult, RestoreError> {
    let fail = || RestoreError::after(ErrorCode::AuthorityContract);
    let exceeded = || RestoreError::after(ErrorCode::LimitExceeded);
    let allocation = || RestoreError::after(ErrorCode::AllocationFailed);
    if values.0.len() != plan.occurrences.len() {
        return Err(fail());
    }
    let mut sizes = Vec::new();
    sizes
        .try_reserve_exact(plan.fields.len())
        .map_err(|_| allocation())?;
    sizes.extend(plan.fields.iter().map(|f| f.text.len()));
    for (occurrence, value) in plan.occurrences.iter().zip(&values.0) {
        let size = &mut sizes[occurrence.field];
        *size = size
            .checked_sub(occurrence.range.len())
            .and_then(|n| n.checked_add(value.len()))
            .ok_or_else(exceeded)?;
    }
    let mut total = 0usize;
    for size in &sizes {
        if *size > plan.limits.field_output_bytes {
            return Err(exceeded());
        }
        total = total.checked_add(*size).ok_or_else(exceeded)?;
    }
    // Saturation safely means all addressable output fits this mathematical ratio.
    if total > plan.limits.output_bytes
        || total > plan.input_bytes.saturating_mul(plan.limits.expansion_ratio)
    {
        return Err(exceeded());
    }
    let mut fields = Vec::new();
    fields
        .try_reserve_exact(plan.fields.len())
        .map_err(|_| allocation())?;
    // Reserve all outputs before copying any plaintext.
    for size in sizes {
        let mut field = String::new();
        field.try_reserve_exact(size).map_err(|_| allocation())?;
        fields.push(field);
    }
    let mut index = 0;
    for (field_index, field) in plan.fields.iter().enumerate() {
        let mut cursor = 0;
        while let Some(occurrence) = plan.occurrences.get(index) {
            if occurrence.field != field_index {
                break;
            }
            fields[field_index].push_str(&field.text[cursor..occurrence.range.start]);
            fields[field_index].push_str(&values.0[index]);
            cursor = occurrence.range.end;
            index += 1;
        }
        fields[field_index].push_str(&field.text[cursor..]);
    }
    Ok(RestoreResult {
        fields,
        restored: index,
    })
}
