# Phase 5c-S — Compiler Safety Audit (Sandbox Pass)

> **Status:** Planned. Zero dependencies on Z3 or AI. Buildable now.  
> **Prerequisites:** None — AST, borrow checker, and contract data already available.  
> **Target:** Before code executes, the compiler audits every unsafe boundary and produces a structured safety report. Works standalone (`--sandbox`) or enhanced with AI explanations (`--sandbox --ai`).

---

## 1. Philosophy: Deterministic Safety, Optional AI

```
┌──────────────────────────────────────────────────────┐
│                 COMPILER PIPELINE                     │
│                                                      │
│  .xi → [Parse] → [Collect] → [Check] → [Contracts]   │
│                                          │           │
│                          ┌───────────────┤           │
│                          │               │           │
│                    ┌─────▼─────┐   ┌────▼─────┐      │
│                    │ --sandbox │   │  Borrow   │      │
│                    │  (ALWAYS  │   │  Checker  │      │
│                    │   PASS)   │   │           │      │
│                    └─────┬─────┘   └──────────┘      │
│                          │                           │
│              ┌───────────┤                           │
│              │           │                           │
│         ┌────▼────┐ ┌───▼────┐                       │
│         │ Stand-  │ │ --ai   │                       │
│         │ alone   │ │ enhance │                      │
│         │ JSON    │ │ report  │                      │
│         └─────────┘ └────────┘                       │
│                                                      │
│              ↓                                       │
│         [Codegen] → [Binary]                         │
└──────────────────────────────────────────────────────┘
```

**The `--sandbox` pass is always deterministic.** It enumerates unsafe blocks, categorizes their contents, scores severity, and produces a structured report. It never makes an LLM call. It never blocks compilation (unless `--sandbox=strict` is used).

**`--ai` enhancement is optional.** When `--sandbox --ai` is combined, the sandbox findings are injected into the AI prompt alongside any compilation errors, giving the LLM richer context for its hints.

---

## 2. CLI Interface

```
xiomc --sandbox source.xi               # Print safety report to stdout
xiomc --sandbox --ai source.xi          # Report + AI-enhanced explanations
xiomc --sandbox=strict source.xi        # REFUSE to compile if HIGH severity findings
xiomc --sandbox-report=json source.xi   # Output as JSON (.xiom_sandbox.json)
xiomc --sandbox-report=text source.xi   # Output as human-readable text (default)
xiomc --sandbox-report=silent source.xi # Suppress output; only exit code
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No findings, or only LOW severity |
| 1 | MEDIUM severity findings present |
| 2 | HIGH severity findings present |
| 3 | `--sandbox=strict` blocked compilation due to HIGH findings |

---

## 3. Safety Audit Pass — What It Checks

### 3.1 Unsafe Block Enumeration (Always On)

Every `unsafe { ... }` block in the AST is visited. The pass records:
- File, line, column
- Number of statements in the block (size = risk surface)
- Whether it has an associated contract (`requires`/`ensures`)
- Whether the containing function is `pub` (public API = higher risk)

### 3.2 Unsafe Operation Categories

| Category | Detection | Default Severity |
|----------|-----------|-----------------|
| `raw_pointer_deref` | `*ptr` expression inside unsafe | HIGH |
| `raw_pointer_arithmetic` | `ptr + n`, `ptr - n` inside unsafe | HIGH |
| `extern_c_call_without_contract` | `extern "C" fn` called in unsafe without `requires` | HIGH |
| `extern_c_call_unchecked_return` | `extern "C"` return value used directly in arithmetic or comparison without guard | MEDIUM |
| `null_pointer_deref_risk` | Pointer loaded from extern, used without null check | HIGH |
| `type_punning` | `ptr as *T` cast followed by deref of different type | HIGH |
| `large_unsafe_block` | Unsafe block > 10 statements | MEDIUM |
| `unsafe_in_public_api` | Unsafe block in a `pub fn` | MEDIUM |
| `unchecked_array_index` | Raw pointer indexing without bounds guard | MEDIUM |
| `unsafe_block_without_comment` | Unsafe block with no preceding `// SAFETY:` comment | LOW |
| `multiple_unsafe_blocks` | Function with > 3 unsafe blocks | LOW |

### 3.3 What It Does NOT Check (Z3 Territory)

These require value-range analysis and are deferred to Phase 5f:
- Integer overflow in unsafe arithmetic
- Buffer overrun in memcpy (size > allocation)
- Use-after-free (borrow checker handles this already)
- Data race in unsafe concurrent access

---

## 4. Output Format

### 4.1 JSON Schema (`.xiom_sandbox.json`)

```json
{
  "schema_version": 1,
  "compiler_version": "0.47.0",
  "file": "src/vulkan/vulkan_safe.xi",
  "timestamp": "2026-07-18T13:46:00",
  "summary": {
    "total_unsafe_blocks": 12,
    "high_severity": 3,
    "medium_severity": 5,
    "low_severity": 4,
    "public_unsafe_functions": 2,
    "safety_score": "MEDIUM"
  },
  "findings": [
    {
      "id": 1,
      "severity": "HIGH",
      "category": "extern_c_call_without_contract",
      "line": 142,
      "column": 5,
      "function": "vk_allocate_buffer",
      "is_public": true,
      "unsafe_block_size": 8,
      "has_contract": false,
      "description": "extern C call xvk_allocate_buffer has no requires/ensures contract. The returned pointer is used without null check.",
      "suggestion": "Add `requires: size > 0; ensures: result != null;` to the containing function, or wrap the call in a null-check guard."
    }
  ],
  "ai_enhanced": false
}
```

### 4.2 Human-Readable Format (stdout default)

```
═══ XIOM Safety Audit: src/vulkan/vulkan_safe.xi ═══

 12 unsafe blocks  |  3 HIGH  |  5 MEDIUM  |  4 LOW  |  Score: MEDIUM

━━━ HIGH ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[HIGH] line 142: extern_c_call_without_contract
  fn vk_allocate_buffer (pub)
  extern C call xvk_allocate_buffer has no requires/ensures
  contract. Returned pointer used without null check.
  → Add `requires: size > 0; ensures: result != null;`

[HIGH] line 205: raw_pointer_arithmetic  
  fn vk_map_memory (pub)
  Unsafe pointer arithmetic (vk_ptr + offset) without bounds
  check. If offset exceeds allocation, this is OOB access.
  → Wrap in bounds check: `if offset >= allocation_size { return; }`

━━━ MEDIUM ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[MEDIUM] line 89: extern_c_call_unchecked_return
  ...

═══ End of report ═══
```

---

## 5. Safety Score Algorithm

The safety score is a weighted aggregate:

```
score = 0
for each finding:
  score += severity_weight[category] * (1 + unsafe_block_size / 10)
score += 5 * (count of unsafe blocks in public functions)
score += 3 * (count of files with any unsafe blocks)

final_score = 
  "SAFE"    if score == 0
  "LOW"     if score <= 10
  "MEDIUM"  if score <= 30
  "HIGH"    if score <= 60
  "CRITICAL" if score > 60
```

### Severity Weights

| Severity | Weight |
|----------|--------|
| HIGH | 5 |
| MEDIUM | 2 |
| LOW | 1 |

---

## 6. Integration with `--ai`

When `--sandbox --ai` is combined, sandbox findings are injected into the AI prompt:

```
[Safety Audit Context]
File: src/vulkan/vulkan_safe.xi
Total unsafe blocks: 12 (3 HIGH, 5 MEDIUM)
Top finding: extern_c_call_without_contract at line 142
  fn vk_allocate_buffer — extern call without contract, no null check on return

[Compiler Error]
Error Code: T001
File: src/vulkan/vulkan_safe.xi:142
...
```

The LLM now has BOTH the safety findings AND the compilation error context, producing richer hints.

### `--sandbox --ai` Output Enhancement

When AI is active, each finding in `.xiom_sandbox.json` gets an additional `ai_insight` field:

```json
{
  "id": 1,
  "severity": "HIGH",
  "ai_insight": "xvk_allocate_buffer returns a raw pointer. Without a null check or contract, any allocation failure causes undefined behavior. The contract `ensures: result != null` documents this for callers and enables the compiler to prove safety at call sites.",
  "ai_confidence": "HIGH"
}
```

---

## 7. Implementation Plan

### 7.1 — Sandbox Pass Core (3–5 Days)

- New file: `crates/xiom-check/src/sandbox.rs`
- `SandboxPass` struct with `audit(program: &Program) -> SafetyReport`
- AST visitor that walks every `Expr::Unsafe` block
- Category detection via pattern matching on AST node types
- Severity scoring algorithm
- JSON + text output formatters

### 7.2 — CLI Integration (1 Day)

- `--sandbox` flag in `crates/xiomc/src/main.rs`
- `--sandbox-report=json|text|silent`
- `--sandbox=strict` mode (abort on HIGH)
- Exit code mapping (0/1/2/3)

### 7.3 — AI Integration (1–2 Days)

- When `--sandbox --ai` combined, inject sandbox context into AI prompt
- Add `ai_insight` field to findings via LLM call
- Add `ai_confidence` to each finding

### 7.4 — CI/CD Integration (1 Day)

- `.xiom_sandbox.json` output path configurable
- `--sandbox=strict` as CI gate: any HIGH = build fails
- GitHub Actions / GitLab CI example configs

---

## 8. Dependency Chain

```
Phase 5c (Production) ✅ ────────┐
Phase 5c-R (Refactor) ✅ ────────┤
Phase 5c-E (Ecosystem) ✅ ───────┤
                                 ├──→ Phase 5c-S (Safety Audit)
Existing AST ✅ ─────────────────┤     ├ 7.1: Sandbox pass core
Existing Borrow Checker ✅ ──────┤     ├ 7.2: CLI integration
Existing Contracts ✅ ───────────┤     ├ 7.3: AI integration (optional)
                                 │     └ 7.4: CI/CD integration
Phase 5g (AI Pipeline) ──────────┘     (AI-enhanced findings)
```

**Zero dependencies.** The sandbox pass uses only the AST, borrow checker data, and contract metadata — all of which exist today. It can be built and shipped in v0.47.0.

---

## 9. Why This Matters

| Without Sandbox | With Sandbox |
|-----------------|-------------|
| Unsafe code is invisible until crash | Every unsafe boundary is audited before codegen |
| Developer reviews unsafe manually | Compiler enumerates and scores every unsafe site |
| Extern calls are undocumented risks | Missing contracts flagged at compile time |
| AI has no safety context | Sandbox findings enrich AI prompts |
| CI/CD trusts developer vigilance | `--sandbox=strict` gates deployments |

## Reference

- `docs/NAMING_CONVENTIONS.md` — API stability rules (apply to sandbox output schema)
- `docs/AI_PIPELINE.md` — AI integration design
- `tests/ecosystem/test_full.xi` — contracts and unsafe blocks for testing
