use crate::{scanner, Limits, RestoreError, RestoreField, RestoreRequest, TrustedContext};
use std::{fmt, ops::Range};

/// Borrowed canonical token and UTF-8 byte range, ordered by field then offset.
#[derive(Clone)]
pub struct Occurrence<'a> {
    pub(crate) field: usize,
    pub(crate) range: Range<usize>,
    pub(crate) token: &'a str,
}
impl<'a> Occurrence<'a> {
    pub fn field_index(&self) -> usize {
        self.field
    }
    pub fn byte_range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub fn token(&self) -> &'a str {
        self.token
    }
}
impl fmt::Debug for Occurrence<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Occurrence")
            .field("field", &self.field)
            .field("range", &self.range)
            .finish_non_exhaustive()
    }
}
/// Immutable validated request snapshot. Contains no resolved values.
/// Inputs stay borrowed and cannot be mutated while this plan is alive.
pub struct RestorePlan<'a> {
    pub(crate) context: TrustedContext<'a>,
    pub(crate) captures: &'a [&'a str],
    pub(crate) fields: &'a [RestoreField<'a>],
    pub(crate) occurrences: Vec<Occurrence<'a>>,
    pub(crate) input_bytes: usize,
    pub(crate) limits: Limits,
}
impl<'a> RestorePlan<'a> {
    pub fn build(request: &RestoreRequest<'a>, limits: Limits) -> Result<Self, RestoreError> {
        let input_bytes = request.validate(limits)?;
        let mut occurrences = Vec::new();
        for (index, field) in request.fields.iter().enumerate() {
            scanner::scan(field.text, index, limits, &mut occurrences)?;
        }
        Ok(Self {
            context: request.context,
            captures: request.captures,
            fields: request.fields,
            occurrences,
            input_bytes,
            limits,
        })
    }
    pub fn context(&self) -> TrustedContext<'a> {
        self.context
    }
    pub fn captures(&self) -> &'a [&'a str] {
        self.captures
    }
    pub fn fields(&self) -> &'a [RestoreField<'a>] {
        self.fields
    }
    pub fn occurrences(&self) -> &[Occurrence<'a>] {
        &self.occurrences
    }
    pub fn input_bytes(&self) -> usize {
        self.input_bytes
    }
}
impl fmt::Debug for RestorePlan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RestorePlan")
            .field("fields", &self.fields.len())
            .field("occurrences", &self.occurrences.len())
            .finish_non_exhaustive()
    }
}
