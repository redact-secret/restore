# Architecture

## 1. Responsibility

`restore` owns the reconstruction algorithm.

It receives tokenized text plus a trusted restore context, asks an authority implementation to validate and resolve the request, and reconstructs output only after the entire request is eligible.

It does not own mappings or authorization state.

## 2. Boundary with the vault

```text
+------------------------+
| redact-secret-vault    |
|------------------------|
| mapping lifecycle      |
| TTL / revoke           |
| usage budgets          |
| principal / tenant     |
| source provenance      |
| sink / path / purpose  |
| persistence            |
| crypto / key providers |
+-----------+------------+
            |
            | small authority contract
            v
+-----------+------------+
|        restore         |
|------------------------|
| token discovery        |
| request planning       |
| full preflight         |
| reconstruction         |
+------------------------+
```

`restore` should depend on a minimal contract crate/module where necessary, not on a database backend.

## 3. Repository boundary is not process boundary

A native Rust consumer must be able to link:

```text
restore + vault implementation
```

into one process with no serialization.

Remote/service authority adapters may exist later, but must remain optional.

## 4. Restore stages

### 4.1 Parse request structure

Validate:

- field count,
- field size,
- sink identifier,
- purpose identifier,
- capture identifiers,
- structural paths if part of the contract.

No plaintext lookup occurs yet.

### 4.2 Discover token occurrences

Scan fields for the canonical token marker.

The scanner must:

- operate in linear or near-linear time,
- avoid allocating one substring per token,
- record ranges into the original field,
- reject malformed marker-preserving token variants according to the token contract.

### 4.3 Build restore plan

The plan should contain only what reconstruction needs:

- field index/path,
- token identifier,
- occurrence byte range,
- expected capture/source reference if required.

### 4.4 Preflight the entire request

All occurrences are checked before output is constructed.

Preflight may validate:

- token syntax,
- known token identity,
- source/capture membership,
- tenant,
- principal,
- expiry,
- revocation,
- sink/path grant,
- purpose,
- remaining use budget,
- application policy.

A single failure denies the request.

### 4.5 Consume / commit

The authority implementation owns the linearization point.

For persistent backends this may be a conditional transaction.

`restore` must not pretend that reconstruction itself is the transaction.

### 4.6 Reconstruct

Only after a successful authority result may plaintext be used to construct the returned fields.

For each field:

1. calculate final size,
2. allocate once,
3. append untouched source segments,
4. append restored values,
5. return complete field.

No partially restored field is returned.

## 5. Authority contract

The contract must be small enough that:

- in-memory vault,
- persistent vault,
- custom enterprise vault,
- remote authority adapter,

can implement it without exposing storage details to `restore`.

The contract must not expose arbitrary mapping iteration.

## 6. Plaintext lifetime

Resolved plaintext should have the shortest practical lifetime.

Where feasible:

- use owned byte buffers with controlled scope,
- avoid cloning,
- drop values immediately after reconstruction,
- consider zeroizing buffers when it is practical and does not create false security claims.

The project must not claim guaranteed memory erasure.

## 7. Performance requirements

1. One token-discovery pass per field.
2. No per-token reconstruction allocation.
3. Bulk authority calls where the authority supports them.
4. No repeated token parsing after plan creation.
5. No JSON/Serde requirement for native Rust.
6. Borrow tokenized input where possible.
7. Pre-size outputs.
8. Avoid quadratic replacement algorithms.
9. Support static linking and LTO.
10. Benchmark high-token-count workloads.

## 8. Concurrency

Concurrency rules belong primarily to the authority implementation.

`restore` must preserve the authority's semantics for:

- concurrent restore,
- concurrent revoke,
- use-budget consumption,
- expiry,
- cancellation.

If the authority says a grant is no longer valid, reconstruction must not proceed.

## 9. Cancellation

Cancellation is security-sensitive.

A future async API must define:

- whether cancellation can occur before consume,
- whether cancellation can occur after commit,
- whether a committed consume may return no plaintext,
- whether retry is permitted.

No async API should be added until these semantics are explicit.

## 10. Free-text restore

Arbitrary `restore(text)` is not automatically safe.

Structured fields with explicit sink/path grants are the preferred first-class model.

If arbitrary text restore is later supported, it must use the same preflight and authorization requirements.

## 11. Error boundary

Errors may include:

- stable error codes,
- coarse denial categories intended for trusted application code,
- bounded counts.

Errors must not include:

- restored values,
- original values,
- token-to-value mappings,
- encryption data,
- secrets from policy callbacks.

## 12. Public-release gate

Before public release:

- the authority contract must be stable,
- shared conformance tests with `redact-secret-vault` must pass,
- transaction/cancellation semantics must be documented,
- persistent and in-memory integrations must be independently qualified,
- fuzzing must cover token scanning and output reconstruction,
- performance and allocation baselines must exist.

## 13. Initial synchronous implementation

The crate now implements request validation, a borrowed immutable RestorePlan,
bulk RestoreAuthority interaction, and pre-sized one-pass output. `restore_plan`
accepts a prebuilt plan without rescanning; it always preflights anew.
The host supplies tenant/principal/session, sink/purpose, captures, and paths.
Structural string validation is not authentication.

The authority trait hands off exactly one owned string per occurrence following
committed consume, in plan order. The authority is trusted for value association
and authorization; the engine checks completeness and output bounds. Plans cannot
be externally forged or mutated. All output reservations precede copying values;
only a complete result is returned. Checked output failures after consume report
Committed and do not roll back budgets. Preflight cannot reserve a use.

The scanner follows the pinned vault marker grammar with Unicode 17 Cf data,
including Unicode casefold long-s in the marker. It does not normalize source
bytes. Whitespace/control insertions that erase the recognizable marker remain
ordinary text, matching that grammar; this is not arbitrary tamper detection.

[Authority decisions](docs/authority-semantics.md) define race/retry/expiry/budget
and sync abandonment semantics. Actual async and persistent-backend qualification
are absent. [Readiness](docs/readiness.md) separates engine evidence, test-authority
models, and missing real vault support. Native dependencies remain empty.
