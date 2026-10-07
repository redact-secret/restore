# Property and fuzz strategy

The dependency-free public crate uses UTF-8 `&str`; it has no byte/FFI entrypoint.
Raw-byte campaigns reject invalid UTF-8 with `str::from_utf8` before planning.
This does not claim that a lossy decoder detects malformed raw tokens.

Run the deterministic bounded mutation/property corpus (20,000 cases,
seed `0x726573746f7265`) with:

```sh
CARGO_BUILD_JOBS=1 cargo run --offline --release --example property_corpus
CARGO_BUILD_JOBS=1 cargo run --offline --release --example fuzz_smoke
CARGO_BUILD_JOBS=1 cargo test --offline --all-features
```

CI executes the corpus on every PR. Integration tests cover canonical/malformed
markers, Unicode/control tampering, authorization denials, resource bounds,
exact untouched-byte preservation, safe formatting, atomic failure, and
controlled authority interleavings. The corpus uses a simple reference replacement
oracle independent of the reconstruction code. Its synthetic authority is not a
vault qualification profile.

Optional libFuzzer setup requires nightly, cargo-fuzz, and libfuzzer-sys. They
are development tools, not native runtime dependencies:

```sh
CARGO_BUILD_JOBS=1 cargo fuzz run restore -- -max_total_time=300 -max_len=65536 -rss_limit_mb=1024
```

`fuzz/corpus/restore/` contains synthetic canonical, adjacent, Unicode,
truncated, invisible/bidi marker, marker-dense, and invalid-UTF-8 seeds. Replacement sizes, output limits, and incomplete authority handoffs vary with
input bytes, covering expansion and atomic denial. The dependency-free
`fuzz_smoke` example executes the same property body over all seeds, dense
canonical cases, and deterministic raw-byte mutations. The single target exercises strict UTF-8 conversion, scanner, plan
determinism, and
reconstruction with a reference byte-range oracle. Extend campaigns with high
density and expansion-specific inputs/authorities when new contracts are added.
The scheduled workflow runs a bounded campaign sequentially. Reproduce failures
using their saved seed in a disposable local run; review artifacts before sharing.
Never upload private tokenized captures or resolved payloads.

Local execution evidence is recorded in readiness.md. A committed fuzz target or
workflow is not evidence of a completed libFuzzer campaign, and the deterministic
corpus is not equivalent to coverage-guided fuzzing. Neither proves the absence
of all leaks or asymptotic performance bugs. Scan complexity is additionally
reviewed against bounded lookahead and the configured limits.

For parity with the pinned public vault regular expressions and Unicode 17 Cf
classification (requires a Node runtime reporting `process.versions.unicode`
as `17.0`), build the development scanner protocol and compare synthetic cases:

```sh
CARGO_BUILD_JOBS=1 cargo build --offline --release --example scan_lines
node scripts/token-parity.mts
```

The script enumerates every Unicode scalar for Cf and casefold marker variants,
then compares scanner results with the documented vault regexes. This is grammar
conformance only; it neither loads a vault adapter nor qualifies authorization.
`src/format_chars.rs` records the generating Node/Unicode version. When updating
the table, enumerate `String.fromCodePoint(cp)` against `/\p{Cf}/u`, coalesce
contiguous codepoints into inclusive ranges, and rerun this parity command and
Unicode regressions. Never substitute a table from an unrecorded runtime.
