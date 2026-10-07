# Authority conformance and compatibility

The reusable synthetic fixtures and authority in `tests/support/mod.rs` run
through the production engine. `tests/engine.rs` and `tests/semantics.rs` exercise
existence/forgery, marker tampering, capture/session/tenant/principal context,
sink/path/purpose, expiry/revoke/budgets, duplicate/reordered occurrences, mixed
requests, changed policy, request-bound grants, output failure, and commit state.

Denied precommit cases return no restored request and spend no unrelated uses.
Successful consumption counts each duplicate occurrence. Postcommit engine
failure returns no output with Committed state; consumed uses are not refunded.
Error/result/plan/resolved-value formatting tests omit synthetic plaintext.
The library emits no logs, traces, or metrics, but these tests do not qualify
host or authority diagnostic channels.

To run the vectors against another authority, preserve the scenario inputs and
observable assertions, replacing only the support fixture's capture/grant/clock/
revoke hooks. An adapter must document its authority identity, pinned public
contract, trusted context source, budget accounting, linearization, expiry clock,
and committed/indeterminate outcomes. Do not implement policy in the engine or
import sibling private storage/crypto code to make a test pass.

| Profile | Current evidence | Qualification |
| --- | --- | --- |
| Synthetic Rust in-memory authority | Local security/race/resource suites | Engine test fixture only |
| Public vault token grammar | Pinned source and local scanner vectors/parity protocol | Grammar compatibility only |
| Real in-memory `@redact-secret/vault` | Inspected public TypeScript source | Not integrated or qualified |
| Persistent authority contract | Decision, outcome and staged-transaction models | Contract model only |
| Real persistent vault/backend | No Rust adapter or integration run | Not qualified |
| Custom/remote authority | Trait can be implemented | Not qualified |

Pinned sibling revision is `022972391314640e719233b3d9d1f3ad0acb802d`,
`@redact-secret/vault` version `0.1.0-beta.5`. Its public TypeScript API does not
expose the native Rust bulk plan/grant/value contract. It reconstructs fields
before final budget mutation in the inspected in-memory implementation; this
engine consumes before reconstructing and explicitly records postcommit burns.
Its page-local API lacks host tenant/principal/purpose fields; server context
must be qualified separately rather than assumed from local tokens.

Issues #5/#7 and epic #1 cannot meet their real-vault integration acceptance
criteria on a dummy authority alone. Next work needs an agreed public vault-facing
native contract and a selected real authority implementation, followed by shared
conformance and separate persistent qualification. A subprocess/JSON test driver
may compare public APIs as optional tooling but cannot be called a zero-serialization
native integration. No sibling production code was changed by this foundation.
