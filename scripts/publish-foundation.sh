#!/bin/sh
# Run from restore with ordinary Git writes, network access, and authenticated gh.
# Does not claim the unqualified real-vault work is complete.
set -eu
branch=feat/controlled-reconstruction-v01
base=21ac0192914b18ec961d7eb174cef750745152af
if [ "$(git rev-parse HEAD)" != "$base" ]; then
    echo 'Base changed; review current main before publishing this snapshot.' >&2
    exit 1
fi
if ! git diff --cached --quiet; then
    echo 'Existing staged work detected; refusing to include it in this commit.' >&2
    exit 1
fi
CARGO_BUILD_JOBS=1 cargo fmt --all --check
CARGO_BUILD_JOBS=1 cargo clippy --offline --all-targets --all-features -- -D warnings
CARGO_BUILD_JOBS=1 cargo test --offline --all-features
CARGO_BUILD_JOBS=1 cargo run --offline --release --example property_corpus
CARGO_BUILD_JOBS=1 cargo run --offline --release --example fuzz_smoke
# Explicit paths preserve unrelated user files and installation/configuration.
git switch -c "$branch"
git add -- \
    .gitignore \
    AGENTS.md \
    ARCHITECTURE.md \
    README.md \
    SECURITY.md \
    Cargo.toml \
    Cargo.lock \
    .github/workflows/ci.yml \
    .github/workflows/fuzz.yml \
    docs/authority-semantics.md \
    docs/benchmark-baseline.csv \
    docs/benchmarking.md \
    docs/conformance.md \
    docs/fuzzing.md \
    docs/readiness.md \
    examples/benchmark.rs \
    examples/fuzz_smoke.rs \
    examples/property_corpus.rs \
    examples/scan_lines.rs \
    examples/support/mod.rs \
    fuzz/Cargo.toml \
    fuzz/corpus/restore/adjacent \
    fuzz/corpus/restore/canonical \
    fuzz/corpus/restore/dense \
    fuzz/corpus/restore/format \
    fuzz/corpus/restore/invalid-utf8 \
    fuzz/corpus/restore/malformed \
    fuzz/corpus/restore/unicode \
    fuzz/fuzz_targets/case.rs \
    fuzz/fuzz_targets/restore.rs \
    scripts/publish-foundation.sh \
    scripts/token-parity.mts \
    src/authority.rs \
    src/error.rs \
    src/format_chars.rs \
    src/lib.rs \
    src/limits.rs \
    src/plan.rs \
    src/reconstruct.rs \
    src/request.rs \
    src/scanner.rs \
    tests/engine.rs \
    tests/semantics.rs \
    tests/support/mod.rs
git commit -m 'Add bounded synchronous controlled reconstruction foundation'
git push -u origin "$branch"
body=$(mktemp)
trap 'rm -f "$body"' EXIT HUP INT TERM
cat > "$body" <<'BODY'
Implement a dependency-free synchronous Rust reconstruction engine: canonical vault-token scanning, immutable borrowed plans, bulk preflight/consume, and pre-sized atomic one-pass output.

Define authority-owned commit/race/expiry/duplicate-budget semantics and Committed/Indeterminate failures without an async API. Include reusable synthetic conformance, deterministic race and transaction models, shared fuzz/property corpus, Unicode parity tooling, sequential CI, allocation benchmarks, and explicit readiness gates.

Local evidence: formatting, Clippy, 18 integration test executions (17 unique), 20,000 property cases, 10,035 shared fuzz-body smoke cases, 698 Unicode grammar parity cases, and recorded synthetic benchmark/allocation baseline. Real vault overhead, native authority integration, persistent backend and async qualification, MSRV CI, and a coverage-guided campaign remain separate gates.

The TypeScript sibling exposes no documented native Rust bulk authority. #5/#7 and epic #1 remain open for real-vault integration; this PR does not claim production readiness.

Closes #2
Closes #3
Closes #4
Closes #6
Closes #8
Closes #9
Closes #10
BODY
pr=$(gh pr create --repo redact-secret/restore --base main --head "$branch" --title 'Add bounded synchronous controlled reconstruction foundation' --body-file "$body")
echo "$pr"
# A check failure/no checks stops here; never merge an unverified snapshot.
gh pr checks "$pr" --watch --fail-fast --interval 10
sha=$(git rev-parse HEAD)
number=${pr##*/}
gh api --method PUT "repos/redact-secret/restore/pulls/$number/merge" -f merge_method=squash -f sha="$sha"
