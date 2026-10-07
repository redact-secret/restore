#![no_main]
use libfuzzer_sys::fuzz_target;
mod case;
fuzz_target!(|bytes: &[u8]| case::exercise(bytes));
