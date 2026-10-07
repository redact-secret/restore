# Agent instructions

## Purpose and source of truth

Read [README.md](README.md), [ARCHITECTURE.md](ARCHITECTURE.md),
[CONVENTIONS.md](CONVENTIONS.md), and [SECURITY.md](SECURITY.md) before changing
contracts, implementation, tests, or workflows. Read the current issue and its
parent epic for issue-driven work. The README API is conceptual, not frozen.

`redact-secret/restore` is the Rust controlled reconstruction engine for the
Redact Secret ecosystem. It is private and under development. At this update,
there is no Rust crate or executable test suite; #2 establishes that foundation.
Recheck the tree rather than assuming this status remains unchanged.

```text
token scan -> RestorePlan -> authority preflight -> consume / commit
           -> pre-sized one-pass reconstruction -> trusted destination
```

## Ownership and trust

- Restore owns discovery, malformed-token detection, request validation, planning,
  bulk authority interaction, reconstruction, and safe metadata/errors.
- Vault or another authority owns issuance and mappings, capture/source binding,
  tenant/principal checks, sink/path/purpose permissions, current policy, expiry,
  revocation, use budgets, persistence, crypto, and key management.
- Core owns secret/PII detection and irreversible sanitization. Hosts establish
  trusted identity/context and control destination access and runtime lifecycle.
- Token syntax and possession do not authorize release. Tokenized text and
  model/tool claims are untrusted; identity, sink, purpose, path, and capture/source
  context must come from the trusted application/authority boundary.
- Depend only on documented public sibling contracts. Do not copy vault internals
  or add a database, detector, key provider, or mandatory HTTP/JSON/Serde/subprocess
  boundary. Resolve missing contracts explicitly with the vault boundary; a test
  authority is not proof of vault compatibility.

## Implementation invariants

- Prefer safe Rust, borrowed inputs, compact byte-range occurrences, and static
  dispatch where practical. Inspect architecture before introducing dependencies.
- Align scanning with the canonical vault token grammar and record its source and
  pinned revision/version. Do not invent a competing format. Reject malformed
  marker-preserving variants, including Unicode/control-character tampering,
  according to the contract; syntax validation is never authorization.
- Validate structure and ordered, non-overlapping UTF-8 byte ranges before
  authority consumption. Discovery/planning must not look up plaintext. Reject
  invalid UTF-8 at external byte/FFI boundaries if those boundaries are added.
- Keep plans deterministic, immutable where practical, bounded, and free of
  resolved plaintext. Preserve field/path, occurrence, and capture/source identity.
- Submit the complete plan for bulk preflight and consume/resolve. One failed
  occurrence denies the request. Never return partial resolved values or a
  partially reconstructed field/request.
- The authority owns the consume/commit linearization point. Preflight cannot
  guarantee a later grant remains current. Reconstruction does not commit or
  roll back authority state.
- Bound fields, bytes per field, total input bytes, token count/length, output
  expansion, and captures where represented. Use checked size arithmetic and
  stable, bounded, plaintext-safe limit failures.
- Pre-size outputs and append untouched segments and resolved values in one
  ordered pass. Preserve bytes outside token ranges. Avoid repeated replacement,
  token reparsing, per-token substring allocation, and quadratic algorithms.
- Keep plaintext short-lived and avoid cloning. Do not claim guaranteed memory
  erasure. Errors, Debug/Display, panics, logs, traces, metrics, audit metadata,
  snapshots, and benchmark reports must not expose plaintext or value mappings.
- Changes to syntax, authorization ordering, consumption, revocation, expiry,
  budgets, cancellation, or partial failure require explicit tests and architectural
  review. Update relevant architecture/security docs with material behavior changes.

## Concurrency and cancellation gate

Issue #8 is a prerequisite to any public async API. Document decisions and test
supported semantics first. The initial scaffold exposes only the minimum
synchronous API required for end-to-end tests.

Specify the authority linearization point, rollback availability, one-use-token
races, revoke/expiry ordering, atomic multi-token budgets, post-commit reconstruction
failure, and cancellation before/during preflight and after commit. State whether
cancel-after-consume burns a use, what is safe to retry, and how persistent
backends communicate committed or indeterminate outcomes. Do not promise retries
or rollback the authority cannot guarantee. Use deterministic race schedules,
not sleep-based timing assertions.

## Issue routing

Open issues were checked on 2026-10-07. Refresh GitHub before relying on status;
if access fails, state the limitation rather than claiming a fresh check.

- [#1](https://github.com/redact-secret/restore/issues/1): v0.1 engine epic and exit criteria.
- [#2](https://github.com/redact-secret/restore/issues/2): Rust scaffold, CI, limits, minimal sync API.
- [#3](https://github.com/redact-secret/restore/issues/3): canonical scanner and malformed-token detection.
- [#4](https://github.com/redact-secret/restore/issues/4): RestoreRequest / RestorePlan contracts.
- [#5](https://github.com/redact-secret/restore/issues/5): bulk RestoreAuthority preflight/consume contract.
- [#6](https://github.com/redact-secret/restore/issues/6): all-or-nothing one-pass reconstruction.
- [#7](https://github.com/redact-secret/restore/issues/7): shared vault authorization/tampering conformance.
- [#8](https://github.com/redact-secret/restore/issues/8): concurrency, revoke, expiry, budget, cancellation semantics.
- [#9](https://github.com/redact-secret/restore/issues/9): fuzzing, Unicode, resource bounds, leak tests.
- [#10](https://github.com/redact-secret/restore/issues/10): performance, allocation, qualification, release gates.

Settle #3/#4 contracts before depending on their plan representation; coordinate
#5 with #8 before freezing authority semantics. Passing dummy-authority tests or
compilation does not qualify in-memory or persistent vault integrations.

## Local skills

Canonical skills live in `.agents/skills/`. Expose them to Claude using relative
directory symlinks `.claude/skills/<name> -> ../../.agents/skills/<name>`.
Edit canonical files and preserve separately installed tool-managed skills.

- `restore-scaffold`: Rust foundation, CI, limits, dependency surface (#2).
- `restore-token-plan`: canonical scanner and request/plan contracts (#3/#4).
- `restore-authority-semantics`: bulk authority contract and commit/race/cancellation decisions (#5/#8).
- `restore-reconstruct`: atomic one-pass output generation and plaintext lifetime (#6).
- `restore-conformance`: authorization, tampering, fuzz/property, resource and leak tests (#7/#9).
- `restore-release-gates`: benchmarks, allocation evidence, integration qualification and readiness (#10).

## Working practices

Use RTK for shell commands when available. Read `~/.codex/RTK.md` if it exists;
a missing machine-local file is not a blocker. Start context discovery with
`rtk proxy graft map` when available. Use `graft ask --source` for relevant spans,
`graft grep` for exhaustive indexed searches, and `graft callers` for dependencies.
Open truncated spans before editing. If no graph covers this repository, read
canonical docs and use `rg`; parent/sibling graph hits are not local coverage.
Refresh applicable graph data after substantial code changes. Do not install or
upgrade tooling just to edit documentation.

Work on a feature branch when Git writes are available. Preserve staged,
unstaged, and untracked user work; do not change unrelated configuration.
Use English docs, API comments, issue/PR text, and concise imperative commits.
Commit, push, publish, close issues, and merge only within the authorized task.
Existing authorization remains valid; do not repeatedly ask for it. Never merge
your own PR without explicit task authorization.

On this low-memory device, run work and heavy checks sequentially. When the user
requests subagent orchestration, keep at most one worker active and review its
result before the next task. Do not launch parallel builds or fuzz campaigns.

## Verification and completion

Once the Rust scaffold exists, run relevant checks from CONVENTIONS.md:

```text
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Use `CARGO_BUILD_JOBS=1` for memory-intensive Cargo work here. Run actual documented
conformance/fuzz/benchmark checks appropriate to the change. Do not invent
commands or passing results when tooling is absent. For documentation-only edits,
check links, ownership, issue routing, and security claims. Inspect new files and
both staged and unstaged diffs; run `rtk git diff --check` and check new-file
whitespace. Validate skill frontmatter and every Claude symlink's resolution.

Use unmistakably synthetic fixtures only; never collect or publish live secrets,
PII/PHI, production payloads, or private mappings. Report suspected sensitive data
by location/category without echoing it. Follow SECURITY.md for disclosure and
do not contact others without authorization.

Before claiming readiness, distinguish design intent, implemented behavior,
tested behavior, and qualified profiles. Record pinned vault-contract compatibility,
toolchain range, test/fuzz coverage, concurrency semantics, performance/allocation
baselines, and sync/async/free-text/streaming limitations. Keep test, in-memory,
persistent, and custom authority qualification separate. Report edits, checks,
and remaining contract/evidence gaps concisely.
