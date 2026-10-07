//! Development-only grammar parity protocol: one hex UTF-8 field per input line.
use redact_secret_restore::*;
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("synthetic corpus line");
        let bytes: Result<Vec<_>, _> = line
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                std::str::from_utf8(pair)
                    .ok()
                    .and_then(|p| u8::from_str_radix(p, 16).ok())
                    .ok_or(())
            })
            .collect();
        let Ok(bytes) = bytes else {
            println!("err");
            continue;
        };
        let Ok(text) = std::str::from_utf8(&bytes) else {
            println!("err");
            continue;
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
        match RestorePlan::build(&request, Limits::default()) {
            Ok(plan) => println!("{}", plan.occurrences().len()),
            Err(_) => println!("err"),
        }
    }
}
