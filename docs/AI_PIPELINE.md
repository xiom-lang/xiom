# Phase 5g — AI-Assisted Compilation Pipeline

> **Status:** Planned. Depends on Phase 5f (Z3 Static Verification).  
> **Prerequisites:** `--diagnostics=json` ✅ (exists), `--contracts` ✅ (runtime), `--verify` (SMT-LIB) ✅ (exists, needs Z3 backend).  
> **Target:** Closed-loop AI code generation — LLM writes code, compiler validates, errors auto-fed back for correction.

---

## Vision

The XIOM compiler becomes an **active participant** in AI code generation. An LLM writes `.xi` code; the compiler checks it; structured JSON diagnostics are fed back to the LLM; the LLM iterates until the code compiles AND passes contract verification. This creates a **self-correcting OS logic** loop.

```
LLM writes .xi → xiomc --ai --diagnostics=json → JSON errors → LLM fixes → compile passes → contracts verified → binary
```

## What Already Exists (Infrastructure Ready)

| Component | Status | Notes |
|-----------|--------|-------|
| `--diagnostics=json` | ✅ | Structured error output with file/line/col/suggestion |
| `--contracts` | ✅ | Runtime contract enforcement (llvm.trap) |
| `--dump-contracts` | ✅ | Contract index as JSON |
| `--verify` | ✅ (stub) | SMT-LIB generation; needs Z3 backend |
| `--no-contracts` | ✅ | Strip all checks for release builds |

## What Needs Building

### 5g.1 — `--ai` Flag (Days)

A new CLI flag that:
1. Takes an LLM API endpoint + model name (`--ai-endpoint`, `--ai-model`)
2. On compilation error, formats diagnostics as an LLM prompt
3. Sends the prompt to the LLM
4. Receives corrected code, writes it back to the file
5. Re-compiles (loop until success or max iterations)

```
xiomc --ai --ai-endpoint=http://localhost:11434 --ai-model=codellama source.xi
```

### 5g.2 — Contract-Guided Prompt Engineering (Days–Weeks)

When `--contracts` is active and verification fails:
- The JSON diagnostic includes the violated contract clause
- The LLM prompt includes: "The function `divide` has contract `requires: b != 0.0`. Your generated code must satisfy this."
- The LLM is guided to produce code that satisfies the contracts
- The compiler acts as a **proof checker** — the LLM proposes, the compiler disposes

### 5g.3 — Static Verification Loop (Weeks — Requires Z3)

After Phase 5f (Z3) is complete:
- Contract violations become compile-time errors, not runtime traps
- The `--verify` flag produces actual SMT proofs
- The AI loop now targets **proof-carrying code**
- "If it compiles, it's safe" becomes a mathematical guarantee

### 5g.4 — Hot Reload + AI (Weeks — Requires 5e)

For autonomous systems (robotics, game engines, physical therapy):
- The compiler produces hot-reloadable modules
- An AI agent tweaks logic in real-time
- The running system swaps modules without restarting
- Contracts ensure the new logic is safe before activation

---

## Dependency Chain

```
Phase 5e (Incremental/Hot Reload) ─┐
Phase 5f (Z3 Static Verification) ─┤
                                    ├──→ Phase 5g (AI Pipeline)
Existing JSON diagnostics ──────────┘
Existing --contracts (runtime) ─────┘
```

## Why This Matters

| Without AI Pipeline | With AI Pipeline |
|---------------------|-----------------|
| Developer writes every line | LLM generates boilerplate + complex logic |
| Errors require manual inspection | Structured JSON → LLM fixes automatically |
| Contract violations = runtime crash | Contract violations = LLM retry loop |
| One shot compilation | Iterative refinement until proof-carrying |

## Honest Assessment

**This is a legitimate and powerful vision.** The infrastructure (JSON diagnostics, contracts, --verify stub) is already in place. The missing pieces are:

1. **Z3 integration (Phase 5f)** — non-negotiable prerequisite for "zero runtime error" guarantee. Without static verification, the AI loop is just iterating on type errors, not logical correctness.

2. **LLM API integration** — straightforward REST/WebSocket client in Rust. ~200 lines.

3. **Prompt engineering** — the hardest part. Getting an LLM to produce contract-satisfying code requires careful prompt design and likely fine-tuning on XIOM syntax.

4. **Iteration budget** — each AI round-trip costs tokens + latency. The compiler should cache successful compilations and avoid re-verifying unchanged functions.

**Recommendation:** Complete Phase 5f (Z3) first. Then build 5g.1 (basic --ai flag) as an MVP. Add 5g.2 (contract-guided prompts) and 5g.3 (static verification loop) iteratively.

## Reference

See `E:\repos\rust` and `E:\repos\z3.rs` for Z3 integration patterns. See `docs/rust/` for rustc lessons on incremental compilation and diagnostics infrastructure.
