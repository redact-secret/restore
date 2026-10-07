# v0.1 foundation readiness

Observed 2026-10-07. **Private development foundation; not production-ready or
qualified for a real vault release path.** Repository version `0.1.0` is a local
crate version with `publish = false`, not a release announcement.

## Implemented and locally tested

- Dependency-free safe Rust synchronous crate; borrowed request/plan, immutable
  ordered UTF-8 occurrences, checked configurable bounds, and coarse safe errors.
- Canonical `<rsv_[a-z2-7]{26}>` grammar, Unicode 17 Cf marker detection and
  casefold long-s rejection; no plaintext lookup during scan/plan.
- Whole-plan preflight and consume followed by output size checks, exact reserves,
  and ordered one-pass construction. Result/plan/value formatting omits content.
- Opaque request-bound synthetic authority, per-occurrence budgets, current-state
  revalidation, one-use concurrency, revoke/expiry/policy schedules, postcommit
  burns, and definite/indeterminate outcomes.
- Deterministic staged-transaction rollback and synchronous preflight abandonment
  models. These do not qualify actual persistence or an async implementation.
- 18 integration test executions (17 unique tests; one support regression is
  compiled into both integration binaries). Tests include authorization, resource
  limits, Unicode byte preservation, malformed markers, and safe formatting.
- Deterministic property corpus: 20,000 cases with a fixed seed.
- Development scanner parity protocol: 698 cases compared to the pinned public
  vault regexes, enumerating Unicode Cf/casefold variants.
- Shared fuzz property body with bounded replacement expansion, incomplete
  authority handoff, output-limit failures, Unicode/raw bytes, and seed corpus.
- Performance/allocation harness and initial recorded synthetic baseline.

## Evidence and reproducibility

Local host: x86_64 macOS 12.7.6 (21H1320), rustc 1.99.0
(`b940084d7`, 2026-09-28). Node 22.23.1 reports Unicode 17.0.
Release builds use thin LTO and one codegen unit. Builds ran sequentially with
`CARGO_BUILD_JOBS=1`. CPU model/process RSS were not assessed: the execution
sandbox denied process inspection and `time -l`'s `sysctl kern.clockrate` call.
Benchmark executable size: 439,008 bytes (instrumented development harness,
not the size of a production vault-linked consumer).

[Recorded CSV](benchmark-baseline.csv) identifies each workload and iteration
count. It measures planned engine flow including synthetic authority allocations;
see [benchmark methodology](benchmarking.md) for interpretation. The source
fingerprint below binds the manifest/library/harness used for that baseline;
timing is an initial observation, not a production SLA or regression threshold.

Local workflow checks passed on Rust 1.85.0 and stable (1.99.0): formatting, Clippy (`-D warnings`),
all 18 integration test executions in debug and release, 20,000 deterministic
property cases, and 10,035 shared fuzz-body smoke cases. The earlier scanner
parity campaign passed 698 cases.

The declared Rust minimum is 1.85. The [PR #11 Actions run](https://github.com/redact-secret/restore/actions/runs/37699520955)
never started a job: GitHub reported failed account payments or an insufficient
spending limit. Local macOS verification does not qualify the remote Ubuntu
matrix. The workflow remains enabled; account billing must be resolved before
remote checks can execute. The weekly/manual coverage-guided campaign requires
nightly/cargo-fuzz/libfuzzer-sys; cargo-fuzz is not installed locally, so no
libFuzzer campaign is claimed.

## Qualification and remaining gates

| Profile or gate | Status |
| --- | --- |
| Synthetic Rust authority | Locally tested engine fixture |
| Pinned vault token grammar | Local grammar vectors/parity passed |
| Real in-memory vault authority | Not integrated or qualified |
| Persistent outcome/transaction contract | Documented and modeled; no backend qualification |
| Custom or remote authority | Contract available; unqualified |
| Public async/cooperative cancellation | Absent; #8 gate retained |
| Context-free free-text restore | Absent; structured text requires trusted sink/path context |
| Streaming/progressive output | Absent |
| Fuzz/property strategy | Seeded shared target, PR corpus, scheduled bounded campaign; long fuzzing pending |
| Performance/allocation baseline | Synthetic local evidence; real-authority overhead pending |
| Supported toolchain range | 1.85.0 and stable 1.99.0 locally tested; remote Ubuntu blocked |
| License/contribution/vulnerability-reporting release gates | Public-release work remains |

See [conformance and compatibility](conformance.md) and
[authority semantics](authority-semantics.md). Sibling revision is
`022972391314640e719233b3d9d1f3ad0acb802d`, package version `0.1.0-beta.5`.
There is no documented native Rust bulk plan/grant/value integration in that
TypeScript sibling. Its in-memory budget-update ordering differs from this
engine's explicit consume-before-reconstruct contract. Test success cannot
establish that the sibling has accepted or implemented this new boundary.

Issues #5/#7 and epic #1 retain real-vault acceptance gaps. Issues #2/#3/#4/#6/
#8/#9/#10 have foundation implementations and local evidence; remote CI remains
blocked by account billing, and real-vault qualification remains separate. The
engine cannot secure a host that lies about context, leaks output, or implements an unsafe authority.
No guaranteed erasure, recoverable process OOM, exactly-once delivery, receipt
plaintext replay, or policy atomicity across independent services is claimed.

Baseline source SHA-256 (sorted source paths, NUL-delimited path/content):
`209d9a819b59425e3a84c558a426b168a4628893458f140fd35da426931788ca`.
