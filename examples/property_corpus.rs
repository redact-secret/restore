//! Deterministic dependency-free mutation/property campaign; reports only counts.
mod support;
use redact_secret_restore::{
    restore, Limits, RestoreField, RestorePlan, RestoreRequest, TrustedContext,
};
use support::{SyntheticAuthority, TOKEN};
fn main() {
    let context = TrustedContext {
        tenant: "synthetic-tenant",
        principal: "synthetic-principal",
        session: "synthetic-session",
        sink: "synthetic-sink",
        purpose: "synthetic-purpose",
    };
    let fragments = [
        "",
        "plain",
        "한글🦀",
        "\0\u{202e}",
        TOKEN,
        "<rsv_",
        "RSV_",
        "r\u{200b}sv_",
        "r s v _",
    ];
    let mut seed = 0x726573746f7265u64;
    let mut cases = 0;
    for _ in 0..20_000 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let a = fragments[seed as usize % fragments.len()];
        let b = fragments[(seed >> 8) as usize % fragments.len()];
        let text = format!("前{a}中{b}後");
        let fields = [RestoreField {
            path: "/synthetic",
            text: &text,
        }];
        let request = RestoreRequest {
            context,
            captures: &["synthetic-capture"],
            fields: &fields,
        };
        let first = RestorePlan::build(&request, Limits::default());
        let second = RestorePlan::build(&request, Limits::default());
        assert!(first.is_ok() == second.is_ok(), "nondeterministic scanner");
        if let (Ok(first), Ok(second)) = (first, second) {
            assert!(
                first
                    .occurrences()
                    .iter()
                    .map(|o| o.byte_range())
                    .eq(second.occurrences().iter().map(|o| o.byte_range())),
                "nondeterministic plan"
            );
            let output = restore(&request, &mut SyntheticAuthority, Limits::default())
                .expect("canonical synthetic campaign restore");
            let expected = text.replace(TOKEN, "synthetic-value");
            assert!(output.fields()[0] == expected, "byte preservation mismatch");
            let debug = format!("{output:?}");
            assert!(
                !debug.contains("synthetic-value"),
                "unsafe result formatting"
            );
        }
        // Raw-byte input has no public scanner entrypoint; strict conversion rejects bad UTF-8.
        let bytes = seed.to_le_bytes();
        if let Ok(text) = std::str::from_utf8(&bytes) {
            let fields = [RestoreField {
                path: "/bytes",
                text,
            }];
            let request = RestoreRequest {
                context,
                captures: &["synthetic-capture"],
                fields: &fields,
            };
            let _ = RestorePlan::build(&request, Limits::default());
        }
        cases += 1;
    }
    println!("deterministic property corpus: {cases} cases passed; seed=0x726573746f7265");
}
