# Performance and allocation measurement

Run sequentially on a quiet machine:

```sh
CARGO_BUILD_JOBS=1 cargo run --offline --release --example benchmark
wc -c target/release/examples/benchmark
/usr/bin/time -l target/release/examples/benchmark
```

The release profile uses thin LTO and one codegen unit. The matrix spans fields,
input bytes, tokens per field, density, small messages, large documents, and
replacement sizes from 1 to 128 bytes. Twenty warmup requests precede each row;
iterations are included in the CSV. Inputs and replacement bytes are synthetic;
reports contain counts and timing only.

`plan_ns` includes structural validation and scanning. `planned_restore_ns`
measures full execution of a prebuilt immutable plan, including bulk authority,
size checks, allocations, reconstruction, and drops. `resolve_ns` measures
synthetic value allocation inside consume, not external authority latency.
Preflight is a constant-time fixture; authority calls are two per request.
Subtracting resolve time only estimates the remaining reconstruction path,
not an isolated production measurement. Planning is measured separately to
avoid repeated parsing being hidden by authority cost.

The example's System allocator wrapper measures successful allocation/reallocation
calls and requested bytes for planned restore, including synthetic authority
allocations. Setup and plan allocations are excluded from this window. It does
not measure allocator usable size, retained bytes, or peak process RSS. Use the
platform process measurement above for RSS; its baseline includes the harness.
Do not compare allocation counts across different fixture authorities as if they
were engine-only allocations. Each nonempty output has one exact reserve;
resolved-value fixture allocation scales with occurrences.

The only unsafe code is this development example's `GlobalAlloc` forwarding
wrapper. It passes pointers/layouts unchanged to System and counts via atomics,
without allocating in allocator callbacks or dereferencing pointers. The library
forbids unsafe code. Check this wrapper when allocator APIs change; it supplies
measurement evidence, not memory protection. It runs single-threaded here.

Record host, OS, toolchain, source identity, workload, release flags, and output
alongside any baseline. Timing varies with instrumentation and machine load;
this initial baseline is evidence, not a universal latency SLA. Qualify real
vault overhead separately. External network, persistent transaction, and
cross-process costs are absent from this fixture. Regression thresholds need
repeated runs on the same controlled host before becoming release blockers.
