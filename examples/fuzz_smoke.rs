//! Executes the same property body used by libFuzzer without its optional tooling.
#[path = "../fuzz/fuzz_targets/case.rs"]
mod case;
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/restore");
    let mut cases = 0;
    for entry in std::fs::read_dir(root).expect("synthetic seed corpus") {
        let bytes = std::fs::read(entry.expect("synthetic seed entry").path())
            .expect("synthetic seed bytes");
        case::exercise(&bytes);
        cases += 1;
    }
    let token = "<rsv_aaaaaaaaaaaaaaaaaaaaaaaaaa>";
    for count in [0, 1, 2, 64, 1024, 2048, 2049] {
        for prefix in ["", "\0\0\0", "a\0", "🦀"] {
            case::exercise(format!("{prefix}{}", token.repeat(count)).as_bytes());
            cases += 1;
        }
    }
    let mut state = 0x66757a7au64;
    let mut bytes = [0u8; 64];
    for _ in 0..10_000 {
        for byte in &mut bytes {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = state as u8;
        }
        case::exercise(&bytes);
        cases += 1;
    }
    println!("shared fuzz property body: {cases} seed/density/expansion/raw-byte cases passed");
}
