# restore

Controlled reconstruction for reversible anonymization in the Redact Secret ecosystem.

> **Status:** Private development repository. This project is not yet considered production-ready or publicly supported. It is intended to become public after its security, conformance, and performance contracts are validated.

## Purpose

`restore` reconstructs trusted output from text containing reversible tokens.

It is deliberately separate from storage and authorization state.

`restore` is responsible for:

- token discovery,
- token occurrence validation,
- construction of a restore plan,
- full-request preflight,
- all-or-nothing reconstruction,
- one-pass output generation.

It is **not** responsible for:

- detecting secrets or PII,
- issuing tokens,
- storing original values,
- deciding tenant or principal identity,
- defining application authorization policy,
- persisting mappings,
- encrypting records,
- managing keys.

Those responsibilities belong elsewhere in the ecosystem.

## Ecosystem relationship

```text
redact-secret
      |
      v
 anonymizer
      |
      v
redact-secret-vault
      |
      | authorized values / restore grant
      v
   restore
      |
      v
trusted destination
```

The repository should depend on a small restore/vault contract rather than on persistence implementations.

## Design goals

- All-or-nothing restoration.
- No partial plaintext release on failed requests.
- Bulk preflight before reconstruction.
- One-pass output construction.
- Zero mandatory serialization for native Rust.
- Small contract with vault authority.
- No ownership of persistence or crypto.
- No rescanning of unrelated source data.
- Bounded behavior under adversarial token counts.
- Explicit treatment of tokens as capabilities only when the authority contract says so.

## Non-goals

`restore` does **not**:

- replace `redact-secret-vault`,
- implement database adapters,
- hold encryption keys,
- own tenant policy,
- infer whether a caller should receive plaintext,
- perform NER,
- perform secret detection,
- expose arbitrary mapping iteration.

## Core rule

A restore request is atomic at the application-visible level:

> If any token occurrence cannot be restored under the current authority contract, the operation returns no restored plaintext.

The exact consume/commit semantics must match the authority implementation, but reconstruction must never leak a partially restored result.

## Conceptual flow

```text
tokenized fields
      |
      v
discover tokens
      |
      v
validate syntax / occurrence structure
      |
      v
build restore plan
      |
      v
preflight entire request
      |
      v
authority grants + values
      |
      v
consume / commit according to contract
      |
      v
pre-size outputs
      |
      v
one-pass reconstruction
```

## Conceptual API

The API is not yet frozen.

A likely direction is:

```rust
pub struct RestoreRequest<'a> {
    pub sink: &'a str,
    pub purpose: &'a str,
    pub fields: &'a [RestoreField<'a>],
}

pub trait RestoreAuthority {
    type Grant;

    fn preflight(
        &self,
        request: &RestorePlan<'_>,
    ) -> Result<Self::Grant, RestoreError>;

    fn consume(
        &mut self,
        grant: Self::Grant,
    ) -> Result<ResolvedValues, RestoreError>;
}

pub fn restore<A: RestoreAuthority>(
    request: &RestoreRequest<'_>,
    authority: &mut A,
) -> Result<RestoreResult, RestoreError>;
```

The final contract may use a different split depending on persistent-vault transaction semantics.

## Performance contract

Repository separation must not impose:

- HTTP,
- JSON,
- Serde,
- subprocesses,
- per-token network calls,
- per-token output allocation.

Native Rust should support in-process static linking and LTO.

The preferred algorithm is:

1. scan token markers once,
2. build one plan,
3. preflight in bulk,
4. resolve/consume in bulk,
5. calculate final output sizes,
6. allocate each output once,
7. reconstruct each output in order.

## Security model

`restore` sits on a plaintext release path and must therefore be treated as security-sensitive.

Token presence alone does not authorize restoration.

The authority layer must establish:

- source/capture provenance,
- tenant,
- principal,
- sink,
- structural path where applicable,
- purpose,
- expiry,
- revocation,
- use budget.

See [SECURITY.md](SECURITY.md) and [ARCHITECTURE.md](ARCHITECTURE.md).

## Repository status

Before public release, this repository should have:

- a stable authority contract,
- conformance tests shared with `redact-secret-vault`,
- malformed-token tests,
- cross-session/cross-tenant denial tests,
- revocation/expiry/use-budget tests,
- partial-failure tests,
- Unicode token-tampering tests,
- fuzz tests for token scanning and reconstruction,
- large-field and many-token benchmarks,
- documented behavior for cancellation and persistent commits.

## License

A license should be added before the repository becomes public.
