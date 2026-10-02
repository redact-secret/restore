# Conventions

## 1. Core principles

This repository favors:

- deny-by-default restoration,
- explicit authority contracts,
- atomic application-visible results,
- safe Rust,
- bulk validation,
- low allocation,
- benchmark-backed optimization,
- small dependency surfaces.

## 2. Language

Primary implementation language: Rust.

Documentation, issue titles, commit messages, and public API comments should be in English.

## 3. Standard checks

Expected baseline:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Additional fuzz/conformance jobs should be added as the workspace is scaffolded.

## 4. Public API discipline

Do not expose storage implementation details.

Public restore APIs should describe:

- requests,
- tokenized fields,
- authority interaction,
- results,
- safe errors.

They should not expose:

- database rows,
- encryption envelopes,
- internal key identifiers,
- arbitrary mapping iteration.

## 5. Performance-sensitive code

Prefer:

- borrowed tokenized input,
- compact token occurrence records,
- bulk preflight,
- pre-sized output buffers,
- one ordered reconstruction pass,
- generic/static dispatch where practical.

Avoid:

- repeated `String::replace`,
- one allocation per token,
- per-token JSON calls,
- reparsing already validated tokens,
- per-token database calls when bulk/transactional authority methods are available.

## 6. Cross-repository dependencies

Depend only on documented public contracts.

Do not import private modules from `redact-secret-vault`.

If a required restore contract does not exist, define it explicitly with the vault repository rather than copying internal code.

## 7. Security-sensitive changes

Changes affecting:

- authorization ordering,
- token syntax,
- consume semantics,
- revocation,
- expiry,
- use budgets,
- cancellation,
- partial failure,

require explicit tests and architectural review.

## 8. Commit messages

Prefer concise imperative messages:

```text
Add bulk restore preflight
Reject malformed marker-preserving tokens
Pre-size reconstructed fields
Define cancel-after-consume behavior
```

## 9. Tests

Behavior changes require tests.

Important suites:

- unit tests,
- conformance tests,
- integration tests,
- fuzz tests,
- race/concurrency tests,
- resource-bound tests.

## 10. Benchmarks

Track:

- bytes per field,
- token count,
- restore latency,
- allocation behavior,
- output growth,
- authority-call count.

Benchmark full request flows, not only token parsers.

## 11. Errors

Use stable, bounded errors.

Never include plaintext values or mapping contents.

Coarse denial reasons should be exposed only when the contract explicitly permits them for trusted application code.

## 12. Unsafe Rust

Prefer no `unsafe`.

Any future `unsafe` code requires documented invariants, focused review, dedicated tests, and measurable justification.

## 13. Documentation

Architecture and security docs must be updated in the same change when behavior materially changes.

## 14. Public-readiness

Before public release:

- add license,
- add contribution guide,
- enable vulnerability reporting,
- verify examples contain synthetic data only,
- document supported vault versions,
- publish conformance and benchmark commands.
