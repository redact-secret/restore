use crate::{ErrorCode, Limits, Occurrence, RestoreError};
#[path = "format_chars.rs"]
mod format_chars;

const TOKEN_BYTES: usize = 32;
/// Canonical grammar and marker contract pinned to vault 022972391314640e719233b3d9d1f3ad0acb802d.
/// Equivalent to /r\p{Cf}*s\p{Cf}*v\p{Cf}*_/giu for ASCII marker letters.
/// Fixed grammar bounds lookahead; every format run is visited a bounded number of times.
pub(crate) fn scan<'a>(
    text: &'a str,
    field: usize,
    limits: Limits,
    out: &mut Vec<Occurrence<'a>>,
) -> Result<(), RestoreError> {
    let bytes = text.as_bytes();
    let mut offset = 0;
    while offset < bytes.len() {
        let c = text[offset..]
            .chars()
            .next()
            .ok_or_else(|| RestoreError::before(ErrorCode::MalformedToken))?;
        if c == 'r' || c == 'R' {
            let mut cursor = offset + 1;
            let mut marker = true;
            for expected in ['s', 'v', '_'] {
                while let Some(next) = text[cursor..].chars().next() {
                    if !format_chars::is_format(next) {
                        break;
                    }
                    cursor += next.len_utf8();
                }
                match text[cursor..].chars().next() {
                    Some(next)
                        if next == expected
                            || (expected != '_' && next == expected.to_ascii_uppercase())
                            || (expected == 's' && next == '\u{017f}') =>
                    {
                        cursor += next.len_utf8()
                    }
                    _ => {
                        marker = false;
                        break;
                    }
                }
            }
            if marker {
                let start = offset
                    .checked_sub(1)
                    .ok_or_else(|| RestoreError::before(ErrorCode::MalformedToken))?;
                let end = start
                    .checked_add(TOKEN_BYTES)
                    .ok_or_else(|| RestoreError::before(ErrorCode::MalformedToken))?;
                let token = bytes
                    .get(start..end)
                    .ok_or_else(|| RestoreError::before(ErrorCode::MalformedToken))?;
                if !token.starts_with(b"<rsv_")
                    || token[31] != b'>'
                    || !token[5..31]
                        .iter()
                        .all(|b| b.is_ascii_lowercase() || (b'2'..=b'7').contains(b))
                {
                    return Err(RestoreError::before(ErrorCode::MalformedToken));
                }
                if TOKEN_BYTES > limits.token_bytes || out.len() >= limits.occurrences {
                    return Err(RestoreError::before(ErrorCode::LimitExceeded));
                }
                out.try_reserve(1)
                    .map_err(|_| RestoreError::before(ErrorCode::AllocationFailed))?;
                // Exact ASCII grammar proves UTF-8 boundaries at start/end.
                out.push(Occurrence {
                    field,
                    range: start..end,
                    token: &text[start..end],
                });
                offset = end;
                continue;
            }
        }
        offset += c.len_utf8();
    }
    Ok(())
}
