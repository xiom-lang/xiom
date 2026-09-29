// PERF-1 local repro: unsafe-block trampoline cost on AtomicInt load/store.
// Mirrors the benchmark lane's atomic_trampoline.xi isolation (4M pairs).
// Run: xiom -o perf_atomic.exe perf_atomic_trampoline.xi && ./perf_atomic.exe
module perf_atomic_trampoline

use xiom.core;
use xiom.io;
use xiom.sync.atomics;
use xiom.time.instant;

fn main() -> Int {
  let n = 4000000;

  // Plain Int loop (baseline).
  let t0 = instant.instant_to_millis(instant.instant_now());
  var x = 0;
  var i = 0;
  while i < n {
    x = x + 1;
    i = i + 1;
  }
  let t1 = instant.instant_to_millis(instant.instant_now());

  // 4M atomic store+load pairs through the stdlib wrappers (each wraps an
  // `unsafe { xiom_atomic_* }` block -> one trampoline per call).
  var a = atomics.atomic_int_new(0);
  let t2 = instant.instant_to_millis(instant.instant_now());
  var i2 = 0;
  while i2 < n {
    atomics.atomic_store(&mut a, i2);
    let v = atomics.atomic_load(&a);
    x = x + v - v;
    i2 = i2 + 1;
  }
  let t3 = instant.instant_to_millis(instant.instant_now());

  io.println("plain:");
  io.println(core.to_string(t1 - t0));
  io.println("atomic:");
  io.println(core.to_string(t3 - t2));
  io.println("x:");
  io.println(core.to_string(x));
  return 0;
}
