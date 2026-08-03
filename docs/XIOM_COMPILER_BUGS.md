# XIOM Compiler — 2 Bugs Blocking Benchmark Arena

> **Paste into compiler session.** Both bugs reproduce in Docker (Ubuntu 24.04, `target/debug/xiom` binary).  
> Full benchmark framework is at `E:\Projects\AXIOM\xiom-benchmark-chaos`.

---

## Bug 1: Windows Runtime Leak — `GetSystemInfo` on Linux

### Symptom
ALL XIOM compilation fails on Linux/Docker with:
```
error: clang failed with exit code 1
/stdlib/runtime/xiom_runtime.c:4372:9: error: use of undeclared identifier 'SYSTEM_INFO'
  SYSTEM_INFO si;
  ^
/stdlib/runtime/xiom_runtime.c:4373:9: error: call to undeclared function 'GetSystemInfo'
  GetSystemInfo(&si);
  ^
```

### Root Cause
`E:\Projects\AXIOM\stdlib\runtime\xiom_runtime.c` lines ~4372-4374 have Windows-only API calls without `#ifdef` guards:

```c
// Line 4372 — WINDOWS ONLY
SYSTEM_INFO si;
GetSystemInfo(&si);
num_workers = si.dwNumberOfProcessors;
```

`SYSTEM_INFO`, `GetSystemInfo`, and `dwNumberOfProcessors` are Win32 API — they don't exist on Linux.

### Fix
Add `#ifdef _WIN32` guard with a Linux fallback (`sysconf(_SC_NPROCESSORS_ONLN)`):

```c
#ifdef _WIN32
    SYSTEM_INFO si;
    GetSystemInfo(&si);
    num_workers = si.dwNumberOfProcessors;
#else
    num_workers = sysconf(_SC_NPROCESSORS_ONLN);
    if (num_workers < 1) num_workers = 1;
#endif
```

### Affected files
- `E:\Projects\AXIOM\stdlib\runtime\xiom_runtime.c` (~line 4372)
- Possibly also `stdlib/runtime/async_runtime.c` (same pattern check)

### Test
After fix, rebuild XIOM binary (`cargo build`) and run in Docker:
```bash
docker compose down
docker compose build --no-cache
docker compose up
```
Select `systems-speed` profile → Run. All 5 XIOM systems tasks (t1-t5) should compile and pass (not crash).

---

## Bug 2: XIOM Systems-Arena SEGFAULT (v0.53.0 Regression)

### Symptom
XIOM compiles successfully for all 5 systems tasks but **SEGFAULTs at runtime**:
```
compiled: .../solution.xi.arena.out
-- test runs, crashes immediately --
status: FAIL, no metrics captured
```
All 5 tasks fail identically. `xiom-run` (scripting/interpreter mode) works fine — only the standalone compiled path (`xiom --target native -o binary source.xi`) crashes.

### Root Cause
Unknown SEGFAULT in v0.53.0 compiled output. The compiler IR→LLVM codegen is producing invalid native code that crashes on entry.

### Test
```bash
# Inside Docker container
cd /app/tmp/builds/run_*/run_*_xiom_t1-allocator_systems-arena_t0/
# The compiled binary should run and output "OK":
./solution.xi.arena.out
# Expected: "OK" printed to stdout, exit code 0
# Actual: SEGFAULT (no output, exit code 139)
```

Or test directly:
```bash
xiom --release --target native --no-contracts -o /tmp/test_binary reference/systems-arena/t1-allocator.xi
/tmp/test_binary
# Should print "OK" — currently SEGFAULTs
```

### Reference files to test against
- `E:\Projects\AXIOM\xiom-benchmark-chaos\reference\systems-arena\t1-allocator.xi`
- `E:\Projects\AXIOM\xiom-benchmark-chaos\reference\systems-arena\t2-queue.xi`
- `E:\Projects\AXIOM\xiom-benchmark-chaos\reference\systems-arena\t3-hot-reload.xi`
- `E:\Projects\AXIOM\xiom-benchmark-chaos\reference\systems-arena\t4-packet.xi`
- `E:\Projects\AXIOM\xiom-benchmark-chaos\reference\systems-arena\t5-btree.xi`

All 5 should compile with `xiom --release --target native --no-contracts -o binary source.xi` and run to output `OK`.

---

## Build & Docker Instructions

1. Fix the bugs in XIOM compiler source
2. Build: `cargo build` (produces `target/debug/xiom`)
3. Copy binary to Docker-visible path (if needed):  
   `cp target/debug/xiom /mnt/e/Projects/AXIOM/target/debug/xiom`
4. Dockerfile at `E:\Projects\AXIOM\xiom-benchmark-chaos\Dockerfile` line 79 copies from `target/debug/xiom`
5. Rebuild and test:
```bash
docker compose down
docker compose build --no-cache
docker compose up
```
6. In dashboard, select `systems-speed` → Run → Verify all XIOM trials show PASS with metrics
