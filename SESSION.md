# XIOM Session Handoff — 2026-08-13 01:10 (compiler session — handoff: everything verified & committed)

> **Context-full handoff.** The compiler session has COMPLETED its work; the next
> session continues from a clean tree. Read this file, then run
> `./test_summary.ps1 -Fast` to re-verify, and answer the two OPEN QUESTIONS at
> the bottom (they are for the NEXT session — do not lose them).

## ✅ Completed this stretch (all committed on `feat/architect`)

| Area | Result |
|------|--------|
| BUG 1–24 (all numbered compiler bugs) | **ALL FIXED** — see docs/COMPILER_BUGS.md |
| BUG 25 wave-3 (12 findings) | **#1/#2/#3/#5/#8/#11 FIXED**, #4/#6/#7 verified-fixed, #9 design, #12 benign, **#10 (crypto) still OPEN** |
| NOTE 4 (module-qualified enum variants) | **FIXED** (`bigfloat.Down` works) |
| BUG 26 (secure numeric policy) | **IMPLEMENTED**: Int↔Float mixing requires explicit `as` (Rust-style); int literals may adopt float; same-family widening stays auto; checker resolves nested Vec elem types + base-Vec methods |
| Labeled loops | **IMPLEMENTED**: `@label: while …` + `break @label;` / `continue @label;` |
| BUG 27 (debug intrinsics) | **IMPLEMENTED**: `assert(cond[, "msg"])`, `dbg!(expr)`, `todo!()`, `unimplemented!()`, `debugger;` — all yield to user fns with the same name; `debugger;` calls runtime `xiom_debugger_break` (no-op without an attached debugger) |

**Key commits:** `dd6a31cd` (numeric policy + labeled loops + debug intrinsics) ·
`4c439e6a` (NOTE 4 + BUG 25 #2 reachability) · `1a2d132a` (docs) · plus the earlier
BUG 12–24 batch (`2ae300fd` … `77a67a01`).

## Verification state (current)
- **31/31 regression sweep** (tests/regression/m37_*.xi) R=0 — includes the 3 new
  feature tests: `m37_numeric_policy`, `m37_labeled_loops`, `m37_debug_intrinsics`
- checker 178/178 · parser 96/96 · lexer 18/18 · ctfe 96/96 · codegen-unit 10/10 ·
  verifier 27/27 · stdlib-compile **40/40** · formatter 79/79 · scripting 34/34 ·
  script-diff 15/15 · integration 128/128 · robustness 63/63 · jit 5/5 · display 5/5
- Workspace `cargo build --workspace`: **zero warnings**
- Fast suite (00:43 run): **1102/12/1** — all 12 failures are the parallel stdlib
  session's IN-FLIGHT work (lsp 1, mcp-server 3, diff 2 [documented ignore +
  their selfhost file vs T002], stdlib-exec 6 [moved/renamed smokes + transient
  mid-run commit — re-verified passing after]). **Zero compiler regressions.**

## ⚠️ Workflow rules (IMPORTANT for the next session)
- The **parallel stdlib session** owns `stdlib/xiom/**` (except `stdlib/runtime/*.c`),
  examples/stdlib_smoke, and selfhost. They commit to the SAME `feat/architect`
  branch and rebuild `target/debug/xiom.exe` constantly. The stdlib is being
  REBUILT from scratch — expect stdlib smoke/suite failures until it lands.
- **ALWAYS use the ISOLATED binary** for compiler verification:
  ```powershell
  $env:CARGO_TARGET_DIR="$env:TEMP\kilo\tgt_iso"; cargo build -p xiom
  Remove-Item Env:CARGO_TARGET_DIR
  $xiom = "$env:TEMP\kilo\tgt_iso\debug\xiom.exe"
  ```
  Check its timestamp after parallel-session commits (they may land mid-run).
- The e2e harness hardcodes `target/debug/xiom.exe` — e2e runs against the
  parallel session's possibly-stale binary; verify via the isolated binary +
  exact-invocation replication instead.
- The user's workflow: stdlib-session smoke tests that find compiler gaps go into
  `docs/COMPILER_BUGS.md` for the compiler session. Compiler-side test lists
  (`crates/xiom-codegen/tests/stdlib_tests.rs`, `stdlib_execution_tests.rs`) need a
  sync AFTER their stdlib layout freezes — do NOT chase mid-rewrite.

## ⏳ Remaining (compiler session, low priority)
1. **BUG 25 #10**: `xiom.crypto` — `use of undefined value '@_pkcs7_pad'` link
   issue (private fn body emission vs bare-symbol resolution) + pure-XIOM SHA-256
   correctness. PRE-EXISTING; reproduced (p_crypto2 probe); needs a dedicated
   investigation session.
2. **fast suite re-run** after the parallel session stabilizes — the stdlib-exec
   count should drop back to their in-flight baseline.
3. **stdlib_tests.rs + stdlib_execution_tests.rs path sync** — one commit after
   their layout freezes.

## 🔧 Compiler behavior the next session should know (documented in COMPILER_BUGS.md)
- **Numeric policy**: `var f: Float64 = int_var;` / `d + int_var` / `d > int_var`
  now ERROR with "convert explicitly with `as`" (int LITERALS are fine).
- **Ref-of-ref guard**: `&x` where `x` is already a `&T` param is a compile error
  (caught real stdlib typos: bigfloat `&base`, spline `&xs`).
- **Ambiguity**: bare fns exported by multiple imported modules → T001 error.
- **Private fns**: never re-exported by `use` (visibility gate).
- **Debug intrinsics**: `assert`/`dbg`/`todo`/`unimplemented`/`debugger` are
  reserved builtins ONLY when no user fn with the name is registered; `dbg!` prints
  `[dbg] <value>` and returns the value; assert violations exit 1 with the message.
- **Labeled loops**: syntax `@label: while …` and `break @label;`.

---

## ❓ OPEN QUESTIONS — for the NEXT session to answer (do NOT lose these)

The user asked these two questions; the compiler session's context window is full
so they are handed over UNANSWERED. Both are language-design/security questions —
the next session should research (incl. docs/ROADMAP.md Phase 5c-E + the stdlib
session's conventions) and answer/implement with the user's confirmation:

**Q1 — `docs/AI_CONTEXT.md` update.** The file is marked IMMUTABLE (only the
language team may update it). The v0.57+ features (BUG 26 numeric policy,
labeled loops, BUG 27 debug intrinsics: `assert(cond[, msg])`, `dbg!(expr)`,
`todo!()`, `unimplemented!()`, `debugger;`) are NOT yet documented there
(section 2 syntax, section 8.1 core intrinsics — `fn assert(condition: Bool,
msg: Str)` exists but the statement form + `dbg!`/`todo!`/`debugger;` do not).
The next session should update AI_CONTEXT.md (with the user's approval to touch
the immutable spec) to document: the numeric-policy rule (Int↔Float requires
`as`, int literals may adopt float), the `@label:` loop syntax, and the debug
intrinsics with their security semantics. Also the `if / elif / else` rule
(section 2.4 says "Not `else if`" — `else if` is NOW accepted as a desugared
form; decide how to document it).

**Q2 — Security review of the debug intrinsics + broader secure-language gaps.**
The user's exact framing: "are these secure macros? also should we include like
IF DEBUG or something to strip debug code on build? I mean what else for a secure
system programming language we need. I know using macros and not restricting them
on users can pass bad code. we don't want that. Does our language have gaps like
it? or any grey areas."

Things to investigate/answer (do NOT implement without user confirmation):
1. Are `assert`/`dbg!`/`todo!`/`debugger;` secure as builtins? (They are NOT
   user-facing macros — they're compiler intrinsics that yield to user fns with
   the same name; `dbg!`/`assert` print to stderr; `debugger;` no-ops without an
   attached debugger. Consider: is stderr exposure acceptable? Should `assert`
   strip in release? Should there be a `--no-assert`/`--debug-build` flag?)
2. Should XIOM add a compile-time debug-build toggle (e.g. `--debug-build`,
   `#[cfg(debug_assertions)]`-style, or a `comptime`-gated `is_debug_build()`
   intrinsic) to strip `assert`/`dbg!` from release binaries — while ensuring the
   STRIPPING cannot change program semantics in unsafe ways?
3. Broader secure-language gap analysis: what other grey areas exist vs the spec?
   (Candidate areas to review: the numeric policy's int-literal float adoption,
   unchecked index/overflow behavior flags, extern FFI confinement, the
   `--enable-unsafe-direct` cap, `asm()` usage, contract stripping
   (`--no-contracts`) vs debug builds, and whether any spec-conformant construct
   the compiler accepts could allow silent undefined behavior.)
4. Any spec-vs-implementation grey areas the parallel stdlib session should know
   before freezing the API (e.g. `else if` vs `elif` spelling, the debug
   intrinsics naming, the numeric-policy error messages as the sanctioned
   conversion guidance).
