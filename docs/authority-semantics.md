# Authority and commit semantics

Decision for restore #5 and #8, observed 2026-10-07. The initial Rust API is
synchronous. This document defines the engine contract; authority-profile
qualification requires separate integration evidence. A test authority does not
establish compatibility with a production vault.

## Source and compatibility boundary

The inspected sibling revision is
`022972391314640e719233b3d9d1f3ad0acb802d` of `redact-secret/redact-secret-vault`.
Relevant pinned sources are:

- [Canonical token source](https://github.com/redact-secret/redact-secret-vault/blob/022972391314640e719233b3d9d1f3ad0acb802d/packages/vault/src/token.ts).
- [In-memory restore implementation](https://github.com/redact-secret/redact-secret-vault/blob/022972391314640e719233b3d9d1f3ad0acb802d/packages/vault/src/vault.ts).
- [Server authority contract](https://github.com/redact-secret/redact-secret-vault/blob/022972391314640e719233b3d9d1f3ad0acb802d/docs/decisions/define-server-authority-interface.md).
- [Persistent transaction decision](https://github.com/redact-secret/redact-secret-vault/blob/022972391314640e719233b3d9d1f3ad0acb802d/docs/decisions/supersede-persistent-store-contract.md).
- [Persistent specification](https://github.com/redact-secret/redact-secret-vault/blob/022972391314640e719233b3d9d1f3ad0acb802d/docs/specs/persistent-vault.md).

The issued syntax is `<rsv_[a-z2-7]{26}>`, exactly 32 ASCII bytes. The sibling
marker detector recognizes case-insensitive `r`, `s`, `v`, `_` with Unicode Cf
characters between those letters. Syntax proves no authorization. Stronger
marker-tampering rejection must be recorded as a stricter restore profile.

The sibling is TypeScript. It exposes neither a documented native Rust authority
trait nor a bulk plan-to-values API directly usable by this crate. Therefore this
Rust contract is an explicit candidate interoperability boundary, rather than a
claim of qualified native vault support. Optional cross-language conformance
tools may exercise public vault operations; the default Rust engine must not
require serialization, a subprocess, a network, or copied vault internals.

## Request, preflight, and handoff

The host supplies trusted tenant/principal/session, source/capture references,
sink, purpose, and structural paths. Text is untrusted. The engine cannot
authenticate that host context; an authority validates it against its own state.
Every occurrence keeps its field identity and UTF-8 byte range. All structural
and range validation precedes authority interaction. A plan holds no plaintext.

Preflight receives the entire immutable plan. It returns an opaque grant bound
to that plan and authority instance, without plaintext or budget changes. The
grant is preparation, not durable permission. Consume receives that same plan
and grant, rechecks mutable eligibility, and atomically commits all budget
changes or none. A stale, substituted, reused, or foreign grant must fail closed.
Both operations are bulk calls; no arbitrary mapping enumeration is exposed.

Successful consume hands off one value per planned occurrence in deterministic
plan order. Duplicate occurrences remain separate for authorization and use
accounting. Implementations may internally deduplicate value storage, but must
authorize every path and count every occurrence. Two occurrences of a token with
one remaining use deny the whole request, matching the inspected vault behavior.
The engine validates value count, output size, and allocations before releasing
any completed request. Returned values and output wrappers must redact Debug;
errors contain only bounded stable codes and commit state.

## Linearization and races

The authority owns the linearization point: the atomic state change that
consumes all requested occurrences. In-memory authorities serialize this change
with revoke and competing restores. Persistent authorities use one conditional
whole-request transaction, including capture-generation/revision checks and a
conflict with every involved capture's revocation fence. Per-token commits do
not satisfy this contract.

| Schedule | Required result |
| --- | --- |
| Two plans preflight a one-use token, then consume in order | First commit succeeds; second denies with no values. |
| Revoke commits before consume | Consume denies, including after successful preflight. |
| Consume commits before revoke | That release may complete; revoke blocks later consumes. |
| Token expires between preflight and consume | Consume denies; equality with expiry is expired. |
| Any token lacks enough budget at consume | No token budget changes; no plaintext result. |
| Persistent transaction fails before definite commit | Return no values; classify definite noncommit separately from uncertainty. |
| Reconstruction fails after definite commit | Return no output; committed uses remain consumed. |

Expiry uses the authority's clock at its documented commit boundary, not a
model-provided timestamp or the earlier preflight time. Persistent adapters must
document clock/skew guarantees. Application identity and policy services may not
participate in the storage transaction. Adapters must document that residual
policy window and any revision/fencing mitigation; the engine cannot promise
atomicity across independent identity systems.

## Failure state and retries

Consume outcomes distinguish `not committed`, `committed`, and `indeterminate`.
Only a definite successful commit returns resolved values. Definite denial or
precommit failure returns no values and consumes no budget. A committed failure
or lost response may burn uses while returning no output. Indeterminate means
the authority cannot establish whether a commit occurred; it denies release and
the caller must not blindly retry with a fresh operation identifier.

Restore never rolls back authority state. Rollback, if available before commit,
is internal to the authority. Postcommit output-limit or allocation failure is
not a rollback mechanism. Hosts can reduce this risk with conservative limits
and authority-provided size information, but cannot claim transactional delivery.

A request denied before commit may be submitted again under fresh authorization;
success is not guaranteed. Committed operations must not replay plaintext from
receipts. Persistent authority receipt lookup can establish the state change but
does not authorize a second release. An unresolved outcome requires authority
reconciliation. Exactly-once destination delivery is outside this contract.

## Cancellation and the async gate

There is no public asynchronous API or cooperative cancellation parameter in
the initial engine. A synchronous call must run to completion; dropping a caller
thread or terminating a process is not an authority rollback protocol.

Any future cancellable wrapper must preserve these rules:

- Cancellation before preflight performs no authority call and consumes no use.
- Cancellation during preflight releases no values and consumes no use, because
  preflight is side-effect-free with respect to budgets and mappings.
- Cancellation before consume starts may drop the grant without spending uses.
- Cancellation while consume is executing requires an authoritative commit
  outcome; abandoning a local task does not establish noncommit.
- Cancellation after commit may burn every committed use without returning
  plaintext. Values remain scoped and dropped; receipts never replay them.

The sync-only gate remains in place until a future API has deterministic tests
for its actual cancellation mechanics and authority adapters. These decisions
do not qualify an asynchronous implementation.

## Required deterministic evidence

Use controlled stage hooks, barriers, or a sequential scheduler with an injected
clock, rather than sleeps. Exercise both orders of restore/revoke; two preflights
followed by competing one-use consumes; expiry exactly at commit; duplicate and
multi-token budgets; atomic transaction failure; changed policy; precommit
abandonment; committed output failure; and indeterminate receipts. Assert no
partial request/value return, unchanged unrelated budgets on noncommit, and
plaintext-safe errors and formatting. Persistent receipt and transaction tests
at the contract level remain distinct from real backend qualification.
