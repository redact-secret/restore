mod support;
use redact_secret_restore::*;
use support::*;

#[test]
fn preserves_unicode_and_non_token_bytes_with_ordered_ranges() {
    let text = format!("한글🦀{TOKEN}\0{OTHER}끝");
    let fields = [RestoreField {
        path: "a",
        text: &text,
    }];
    let req = request(&fields);
    let plan = RestorePlan::build(&req, Limits::default()).unwrap();
    assert_eq!(plan.occurrences().len(), 2);
    for occurrence in plan.occurrences() {
        assert_eq!(&text[occurrence.byte_range()], occurrence.token());
    }
    assert!(plan.occurrences()[0].byte_range().end <= plan.occurrences()[1].byte_range().start);
    let result = restore(&req, &mut Authority::new(2), Limits::default()).unwrap();
    assert!(
        result.fields() == [format!("한글🦀{SECRET}\0{SECRET}끝")],
        "unicode reconstruction mismatch"
    );
    assert_eq!(result.restored(), 2);
    for debug in [
        format!("{req:?}"),
        format!("{plan:?}"),
        format!("{:?}", plan.occurrences()),
        format!("{result:?}"),
        format!("{:?}", ResolvedValues::new(vec![SECRET.into()])),
    ] {
        assert!(!debug.contains(SECRET));
        assert!(!debug.contains(TOKEN));
    }
}

#[test]
fn malformed_marker_corpus_denies_before_authority() {
    let corpus = [
        TOKEN.to_uppercase(),
        TOKEN[..31].into(),
        TOKEN.replace('a', "8"),
        TOKEN.replace("rsv_", "r\u{200b}s\u{202e}v_"),
        TOKEN.replace("rsv_", "RSV_"),
        TOKEN.replace("rsv_", "rsv_ "),
        format!("{TOKEN}rsv_"),
        "rsv_".into(),
        "rſv_".into(),
        "<rsv_rsv_>".into(),
    ];
    for text in corpus {
        let fields = [RestoreField {
            path: "a",
            text: &text,
        }];
        let mut authority = Authority::new(2);
        assert_eq!(
            restore(&request(&fields), &mut authority, Limits::default())
                .unwrap_err()
                .code,
            ErrorCode::MalformedToken
        );
        assert_eq!(authority.0.lock().unwrap().preflights, 0);
    }
    // The pinned marker contract deliberately does not normalize whitespace.
    for text in ["r s v _", "r\ns\nv_", "ordinary rsv text", "<SECRET_1>"] {
        let fields = [RestoreField { path: "a", text }];
        assert!(RestorePlan::build(&request(&fields), Limits::default())
            .unwrap()
            .occurrences()
            .is_empty());
    }
}

#[test]
fn each_input_and_output_limit_has_stable_commit_state() {
    let fields = [
        RestoreField {
            path: "a",
            text: TOKEN,
        },
        RestoreField {
            path: "b",
            text: OTHER,
        },
    ];
    for limits in [
        Limits {
            fields: 1,
            ..Limits::default()
        },
        Limits {
            field_bytes: 31,
            ..Limits::default()
        },
        Limits {
            input_bytes: 63,
            ..Limits::default()
        },
        Limits {
            occurrences: 1,
            ..Limits::default()
        },
        Limits {
            token_bytes: 31,
            ..Limits::default()
        },
        Limits {
            captures: 0,
            ..Limits::default()
        },
        Limits {
            context_bytes: 1,
            ..Limits::default()
        },
    ] {
        let mut authority = Authority::new(2);
        assert_eq!(
            restore(&request(&fields), &mut authority, limits).unwrap_err(),
            RestoreError::new(ErrorCode::LimitExceeded, CommitState::NotCommitted)
        );
        assert_eq!(authority.0.lock().unwrap().preflights, 0);
    }
    for limits in [
        Limits {
            field_output_bytes: SECRET.len() - 1,
            ..Limits::default()
        },
        Limits {
            output_bytes: SECRET.len() * 2 - 1,
            ..Limits::default()
        },
        Limits {
            expansion_ratio: 0,
            ..Limits::default()
        },
    ] {
        let mut authority = Authority::new(2);
        assert_eq!(
            restore(&request(&fields), &mut authority, limits).unwrap_err(),
            RestoreError::new(ErrorCode::LimitExceeded, CommitState::Committed)
        );
        assert_eq!(authority.used(TOKEN), 1);
        assert_eq!(authority.used(OTHER), 1);
    }
}

#[test]
fn invalid_structure_never_calls_authority() {
    let fields = [
        RestoreField {
            path: "a",
            text: TOKEN,
        },
        RestoreField {
            path: "a",
            text: OTHER,
        },
    ];
    let mut authority = Authority::new(2);
    assert_eq!(
        restore(&request(&fields), &mut authority, Limits::default())
            .unwrap_err()
            .code,
        ErrorCode::InvalidRequest
    );
    assert_eq!(authority.0.lock().unwrap().preflights, 0);
}

#[test]
fn authorization_denials_and_mixed_failure_do_not_spend_uses() {
    let fields = [RestoreField {
        path: "a",
        text: TOKEN,
    }];
    for index in 0..9 {
        let mut authority = Authority::new(1);
        let mut req = request(&fields);
        match index {
            0 => req.context.tenant = "foreign",
            1 => req.context.principal = "foreign",
            2 => req.context.session = "foreign",
            3 => req.context.sink = "foreign",
            4 => req.context.purpose = "foreign",
            5 => req.captures = &["foreign"],
            6 => authority.0.lock().unwrap().policy = false,
            7 => authority.0.lock().unwrap().clock = 10,
            _ => {
                authority
                    .0
                    .lock()
                    .unwrap()
                    .entries
                    .get_mut(TOKEN)
                    .unwrap()
                    .revoked = true
            }
        }
        let error = restore(&req, &mut authority, Limits::default()).unwrap_err();
        assert_eq!(error.commit, CommitState::NotCommitted);
        assert_eq!(authority.used(TOKEN), 0);
        assert!(!format!("{error:?} {error}").contains(SECRET));
    }
    for text in [
        format!("{TOKEN}{TOKEN}"),
        format!("{TOKEN}<rsv_cccccccccccccccccccccccccc>"),
    ] {
        let fields = [RestoreField {
            path: "a",
            text: &text,
        }];
        let mut authority = Authority::new(1);
        assert!(restore(&request(&fields), &mut authority, Limits::default()).is_err());
        assert_eq!(authority.used(TOKEN), 0);
    }
}

#[test]
fn deterministic_generated_byte_preservation() {
    for count in 0..64 {
        let prefix = "é🦀\0".repeat(count);
        let text = format!("{prefix}{TOKEN}{prefix}{OTHER}{prefix}");
        let fields = [RestoreField {
            path: "a",
            text: &text,
        }];
        let result = restore(&request(&fields), &mut Authority::new(2), Limits::default()).unwrap();
        assert!(
            result.fields()[0] == format!("{prefix}{SECRET}{prefix}{SECRET}{prefix}"),
            "byte preservation mismatch"
        );
    }
}

#[test]
fn prebuilt_plans_reorder_and_repeat_with_per_occurrence_budgets() {
    let text = format!("{OTHER}{TOKEN}{TOKEN}");
    let fields = [RestoreField {
        path: "b",
        text: &text,
    }];
    let plan = RestorePlan::build(&request(&fields), Limits::default()).unwrap();
    let mut authority = Authority::new(2);
    let result = restore_plan(&plan, &mut authority).unwrap();
    assert!(
        result.fields()[0] == SECRET.repeat(3),
        "duplicate reconstruction mismatch"
    );
    assert_eq!(authority.used(TOKEN), 2);
    assert_eq!(authority.used(OTHER), 1);
    assert!(restore_plan(&plan, &mut authority).is_err());
    assert_eq!(authority.used(OTHER), 1);
}

#[test]
fn wrong_path_and_duplicate_captures_fail_without_release() {
    let fields = [RestoreField {
        path: "foreign",
        text: TOKEN,
    }];
    let mut authority = Authority::new(1);
    assert_eq!(
        restore(&request(&fields), &mut authority, Limits::default())
            .unwrap_err()
            .code,
        ErrorCode::Denied
    );
    assert_eq!(authority.used(TOKEN), 0);
    let mut req = request(&fields);
    req.captures = &["capture", "capture"];
    assert_eq!(
        RestorePlan::build(&req, Limits::default())
            .unwrap_err()
            .code,
        ErrorCode::InvalidRequest
    );
}

#[test]
fn format_character_tampering_and_density_remain_bounded() {
    for c in [
        '\u{00ad}',
        '\u{061c}',
        '\u{200b}',
        '\u{200d}',
        '\u{202e}',
        '\u{2066}',
        '\u{feff}',
        '\u{e0001}',
    ] {
        let text = TOKEN.replace("rsv_", &format!("r{c}s{c}v{c}_"));
        let fields = [RestoreField {
            path: "a",
            text: &text,
        }];
        assert_eq!(
            RestorePlan::build(&request(&fields), Limits::default())
                .unwrap_err()
                .code,
            ErrorCode::MalformedToken
        );
    }
    let text = TOKEN.repeat(2048);
    let fields = [RestoreField {
        path: "a",
        text: &text,
    }];
    let limits = Limits {
        occurrences: 2047,
        ..Limits::default()
    };
    assert_eq!(
        RestorePlan::build(&request(&fields), limits)
            .unwrap_err()
            .code,
        ErrorCode::LimitExceeded
    );
    let limits = Limits {
        occurrences: 2048,
        ..Limits::default()
    };
    assert_eq!(
        RestorePlan::build(&request(&fields), limits)
            .unwrap()
            .occurrences()
            .len(),
        2048
    );
}
