<!-- Copyright (c) 2026 Eleftherios Notas and XIOM Foundation -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

Continue from V61 handoff. Full context in BENCHMARK-SESSION.md.

## Remaining Issues to Fix

### 1. Safety probe still produces no JSON in Docker
"Safety probe did not produce valid JSON output" for ALL languages. Binary compiles and runs but stdout has no JSON. Likely seccomp restriction blocking fork() in Docker Desktop. Need to either:
- Test with `--security-opt seccomp=unconfined` in docker-compose
- Or rewrite safety probe to use threads instead of fork()
- TCO fix deployed (dummy[0] after recursive call) but may need verification

### 2. Zig t5-btree still FAIL in systems + contracts arenas
@intCast fix deployed (pure usize arithmetic in btreeInsertNonfull). Reference files at reference/*/t5-btree.zig. Verify after Docker rebuild -- check if Zig 0.13.0 has union/zeroes compatibility issue.

### 3. XIOM-run scripting (JIT variants) FAIL
- JIT: `--lazy` removed, now `xiom run --jit --cache`
- AOT: linker error `xiom_str_len` undefined -- needs runtime rebuild in XIOM repo
- Need Docker rebuild with new XIOM binary

### 4. Ada/SPARK contracts (t1,t2,t4,t5) FAIL
SPARK-incompatible constructs in reference files. Need SPARK-compliant rewrites or change contracts arena to use standard Ada mode.

### 5. Pass-quality validation for LLM tests
Need plan to prevent LLM from passing with trivial code like `int main() { return 0; }`.

## Docker Rebuild Needed
```powershell
cd E:\Projects\AXIOM\xiom-benchmark-chaos
docker compose down
docker compose build --no-cache
docker compose up
```

## Quick Validation
```bash
cd dashboard && npx tsc --noEmit && npx vite build
```

## Environment
- Working directory: E:\Projects\AXIOM
- Benchmark root: E:\Projects\AXIOM\xiom-benchmark-chaos
- Dashboard: E:\Projects\AXIOM\xiom-benchmark-chaos\dashboard
- Branch: feat/architect
