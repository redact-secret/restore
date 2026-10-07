//! Single-thread synthetic measurement. Unsafe is isolated to this development
//! harness's System allocator delegation, never used in the library.
use redact_secret_restore::*;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
static COUNT: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
struct Counter;
// SAFETY: Every operation forwards the exact pointer/layout to System. Counters
// allocate nothing, never dereference memory, and use atomics. Successful alloc/
// realloc requests are counted; this is requested capacity, not process RSS.
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            COUNT.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            COUNT.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let ptr = unsafe { System.realloc(ptr, layout, size) };
        if !ptr.is_null() {
            COUNT.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(size, Ordering::Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}
#[global_allocator]
static ALLOCATOR: Counter = Counter;
struct Authority<'a> {
    value: &'a str,
    calls: usize,
    elapsed_ns: u128,
}
impl RestoreAuthority for Authority<'_> {
    type Grant = usize;
    fn preflight(&self, plan: &RestorePlan<'_>) -> Result<usize, RestoreError> {
        Ok(plan.occurrences().len())
    }
    fn consume(
        &mut self,
        count: usize,
        _: &RestorePlan<'_>,
    ) -> Result<ResolvedValues, RestoreError> {
        let t = Instant::now();
        let values = ResolvedValues::new((0..count).map(|_| self.value.to_owned()).collect());
        self.calls += 2;
        self.elapsed_ns += t.elapsed().as_nanos();
        Ok(values)
    }
}
fn main() {
    let token = "<rsv_aaaaaaaaaaaaaaaaaaaaaaaaaa>";
    let context = TrustedContext {
        tenant: "synthetic",
        principal: "synthetic",
        session: "synthetic",
        sink: "synthetic",
        purpose: "synthetic",
    };
    println!("fields,padding,tokens_per_field,replacement_bytes,input_bytes,iterations,plan_ns,planned_restore_ns,resolve_ns,allocation_calls,allocation_requested_bytes,authority_calls");
    for (field_count, padding, tokens, replacement_bytes, iterations) in [
        (1, 0, 0, 16, 10000),
        (1, 32, 1, 16, 10000),
        (8, 128, 8, 16, 1000),
        (1, 0, 1024, 1, 500),
        (1, 0, 1024, 128, 500),
        (1, 1000000, 1, 16, 50),
        (64, 256, 16, 64, 100),
    ] {
        let text = format!("{}{}", "x".repeat(padding), token.repeat(tokens));
        let paths: Vec<String> = (0..field_count)
            .map(|i| format!("/synthetic/{i}"))
            .collect();
        let fields: Vec<_> = paths
            .iter()
            .map(|path| RestoreField { path, text: &text })
            .collect();
        let request = RestoreRequest {
            context,
            captures: &["synthetic-capture"],
            fields: &fields,
        };
        let replacement = "x".repeat(replacement_bytes);
        let mut authority = Authority {
            value: &replacement,
            calls: 0,
            elapsed_ns: 0,
        };
        let limits = Limits::default();
        let plan = RestorePlan::build(&request, limits).expect("bounded benchmark plan");
        for _ in 0..20 {
            black_box(restore_plan(&plan, &mut authority).expect("benchmark warmup"));
        }
        let t = Instant::now();
        for _ in 0..iterations {
            black_box(RestorePlan::build(black_box(&request), limits).expect("benchmark plan"));
        }
        let plan_ns = t.elapsed().as_nanos() / iterations;
        authority.calls = 0;
        authority.elapsed_ns = 0;
        COUNT.store(0, Ordering::Relaxed);
        BYTES.store(0, Ordering::Relaxed);
        let t = Instant::now();
        for _ in 0..iterations {
            black_box(restore_plan(black_box(&plan), &mut authority).expect("benchmark restore"));
        }
        let restore_ns = t.elapsed().as_nanos() / iterations;
        let allocations = COUNT.load(Ordering::Relaxed) / iterations as usize;
        let allocated_bytes = BYTES.load(Ordering::Relaxed) / iterations as usize;
        println!("{field_count},{padding},{tokens},{replacement_bytes},{},{iterations},{plan_ns},{restore_ns},{},{allocations},{allocated_bytes},{}",
            text.len()*field_count, authority.elapsed_ns / iterations, authority.calls / iterations as usize);
    }
}
