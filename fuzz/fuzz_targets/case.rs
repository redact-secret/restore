use redact_secret_restore::*;
struct Authority {
    replacement: String,
    wrong_count: bool,
}
impl RestoreAuthority for Authority {
    type Grant = usize;
    fn preflight(&self, plan: &RestorePlan<'_>) -> Result<usize, RestoreError> {
        Ok(plan.occurrences().len())
    }
    fn consume(
        &mut self,
        count: usize,
        _: &RestorePlan<'_>,
    ) -> Result<ResolvedValues, RestoreError> {
        Ok(ResolvedValues::new(
            (0..count + usize::from(self.wrong_count))
                .map(|_| self.replacement.clone())
                .collect(),
        ))
    }
}
pub fn exercise(bytes: &[u8]) {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return;
    };
    let fields = [RestoreField {
        path: "/synthetic",
        text,
    }];
    let request = RestoreRequest {
        context: TrustedContext {
            tenant: "synthetic",
            principal: "synthetic",
            session: "synthetic",
            sink: "synthetic",
            purpose: "synthetic",
        },
        captures: &["synthetic-capture"],
        fields: &fields,
    };
    let output_bound = usize::from(bytes.get(1).copied().unwrap_or(255)) * 4096;
    let limits = Limits {
        field_bytes: 65536,
        input_bytes: 65536,
        occurrences: 2048,
        field_output_bytes: output_bound,
        output_bytes: output_bound,
        ..Limits::default()
    };
    let replacement = "x".repeat(usize::from(bytes.first().copied().unwrap_or(0)) * 16);
    let wrong_count = bytes.get(2).copied() == Some(0);
    let mut authority = Authority {
        replacement,
        wrong_count,
    };
    let first = RestorePlan::build(&request, limits);
    let second = RestorePlan::build(&request, limits);
    assert!(first.is_ok() == second.is_ok());
    if let (Ok(plan), Ok(other)) = (first, second) {
        assert!(plan
            .occurrences()
            .iter()
            .map(|o| o.byte_range())
            .eq(other.occurrences().iter().map(|o| o.byte_range())));
        let mut expected = String::new();
        let mut cursor = 0;
        for occurrence in plan.occurrences() {
            let range = occurrence.byte_range();
            expected.push_str(&text[cursor..range.start]);
            expected.push_str(&authority.replacement);
            cursor = range.end;
        }
        expected.push_str(&text[cursor..]);
        let result = restore(&request, &mut authority, limits);
        if wrong_count {
            assert!(matches!(
                result,
                Err(RestoreError {
                    code: ErrorCode::AuthorityContract,
                    commit: CommitState::Committed
                })
            ));
        } else if expected.len() > output_bound
            || expected.len() > text.len().saturating_mul(limits.expansion_ratio)
        {
            assert!(matches!(
                result,
                Err(RestoreError {
                    code: ErrorCode::LimitExceeded,
                    commit: CommitState::Committed
                })
            ));
        } else {
            assert!(result.expect("bounded valid fuzz plan").fields()[0] == expected);
        }
    }
}
