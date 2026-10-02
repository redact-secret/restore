# Security Policy

## Status

This repository is currently private and under active development.

Any security behavior described here must be treated as a target contract until a release explicitly states that it is implemented and qualified.

## Reporting a vulnerability

Do not put live secrets, PII, vault mappings, tokens tied to real data, encryption keys, database dumps, or customer payloads in issue reports.

Before public release, enable GitHub private vulnerability reporting / Security Advisories and publish the supported reporting path.

## Security sensitivity

`restore` is a plaintext release component.

A defect can expose values that were intentionally hidden before crossing an untrusted boundary.

For that reason:

- token possession must not equal authorization,
- partial restore must not be returned after a failed request,
- authorization must be current at restore time,
- restored values must never enter logs or diagnostic payloads.

## Trust model

The caller-provided model output or tokenized text is untrusted.

Trusted context must come from the application / authority boundary.

Untrusted text must not be allowed to self-declare:

- tenant,
- principal,
- purpose,
- sink,
- authorization,
- capture ownership.

## All-or-nothing rule

The preferred security contract is:

> one invalid or unauthorized token denies the entire restore request.

The implementation must not progressively release plaintext while still validating later tokens.

## Token handling

The implementation must safely handle:

- forged tokens,
- truncated tokens,
- case-changed tokens,
- inserted whitespace,
- invisible Unicode characters,
- bidi/control characters,
- duplicated tokens,
- repeated tokens with limited budgets,
- tokens from another capture,
- literal token-like input.

Token syntax validation is not authorization.

## Authority behavior

`restore` relies on an authority implementation for:

- token existence,
- capture provenance,
- tenant checks,
- principal checks,
- sink/path grants,
- purpose,
- expiry,
- revocation,
- usage budget,
- current application policy.

Failures must deny by default.

## Plaintext handling

Restored values must not appear in:

- errors,
- logs,
- traces,
- metrics,
- panic text,
- audit metadata,
- snapshot fixtures,
- benchmark output.

Where practical, plaintext buffers should be short-lived.

Do not claim complete memory erasure.

## Denial-of-service controls

Bound:

- number of fields,
- bytes per field,
- total bytes,
- token count,
- token length,
- output expansion,
- captures per request.

Token scanning and reconstruction must avoid attacker-controlled quadratic behavior.

## Persistence boundary

This repository must not open a database directly unless the architecture is intentionally changed by a reviewed decision.

Persistence and crypto belong to the vault ecosystem.

## Async and cancellation

Do not add async restore without defining cancellation semantics around the authority's commit/consume point.

A cancel-after-consume scenario can consume a one-time token without returning plaintext. That may be acceptable, but it must be explicit and tested.

## Security testing

Before public release include:

- malformed-token fuzzing,
- reconstruction fuzzing,
- cross-capture denial tests,
- cross-tenant denial tests,
- expired/revoked token tests,
- concurrent revoke/restore tests,
- use-budget race tests,
- no-plaintext-in-error tests,
- token-tampering Unicode corpus,
- cancellation tests for async implementations.

## Disclosure discipline

Documentation must clearly distinguish:

- design intent,
- implemented controls,
- tested controls,
- qualified runtime profiles.
