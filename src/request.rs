use crate::{ErrorCode, Limits, RestoreError};
use std::collections::BTreeSet;
use std::fmt;

/// Supplied by the authenticated host, never derived from model-provided claims.
/// String validation does not authenticate these identifiers.
#[derive(Clone, Copy)]
pub struct TrustedContext<'a> {
    pub tenant: &'a str,
    pub principal: &'a str,
    pub session: &'a str,
    pub sink: &'a str,
    pub purpose: &'a str,
}
#[derive(Clone, Copy)]
pub struct RestoreField<'a> {
    pub path: &'a str,
    pub text: &'a str,
}
pub struct RestoreRequest<'a> {
    pub context: TrustedContext<'a>,
    pub captures: &'a [&'a str],
    pub fields: &'a [RestoreField<'a>],
}
impl fmt::Debug for RestoreRequest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RestoreRequest")
            .field("fields", &self.fields.len())
            .field("captures", &self.captures.len())
            .finish_non_exhaustive()
    }
}
impl RestoreRequest<'_> {
    pub(crate) fn validate(&self, limits: Limits) -> Result<usize, RestoreError> {
        let invalid = || RestoreError::before(ErrorCode::InvalidRequest);
        let exceeded = || RestoreError::before(ErrorCode::LimitExceeded);
        if self.fields.len() > limits.fields || self.captures.len() > limits.captures {
            return Err(exceeded());
        }
        let mut context_bytes = 0usize;
        let mut identifier = |id: &str| -> Result<(), RestoreError> {
            if id.is_empty() || id.trim() != id || id.chars().any(char::is_control) {
                return Err(invalid());
            }
            context_bytes = context_bytes.checked_add(id.len()).ok_or_else(exceeded)?;
            if context_bytes > limits.context_bytes {
                return Err(exceeded());
            }
            Ok(())
        };
        for id in [
            self.context.tenant,
            self.context.principal,
            self.context.session,
            self.context.sink,
            self.context.purpose,
        ] {
            identifier(id)?;
        }
        if self.captures.is_empty() {
            return Err(invalid());
        }
        let mut captures = BTreeSet::new();
        for capture in self.captures {
            identifier(capture)?;
            if !captures.insert(capture) {
                return Err(invalid());
            }
        }
        let mut paths = BTreeSet::new();
        let mut input = 0usize;
        for field in self.fields {
            identifier(field.path)?;
            if !paths.insert(field.path) {
                return Err(invalid());
            }
            if field.text.len() > limits.field_bytes {
                return Err(exceeded());
            }
            input = input.checked_add(field.text.len()).ok_or_else(exceeded)?;
            if input > limits.input_bytes {
                return Err(exceeded());
            }
        }
        Ok(input)
    }
}
