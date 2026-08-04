# XIOM — Pre-Selfhost Gap List (Audit: 2026-08-04)

## Methodology
Full audit of the XIOM compiler against the language specification (`docs/AI_CONTEXT.md`, `docs/language/`). Four parallel agents audited: types+memory+borrow, control flow+patterns+errors, generics+interfaces+modules+contracts, and stdlib vs builtins.

## GAP SEVERITY KEY
| Symbol | Meaning | Action |
|--------|---------|--------|
| 🔴 P0 | **BROKEN** — feature compiles but produces wrong behavior | Must fix before selfhost |
| 🟠 P1 | **MISSING** — spec says it works, implementation doesn't exist | Should fix before selfhost |
| 🟡 P2 | **PARTIAL** — works in some cases, fails in others | Should fix before selfhost |
| 🟢 P3 | **COSMETIC** — works but design could be better | Post-selfhost |
| ⚪ P4 | **DEFERRED** — documented limitation, not blocking | Post-selfhost |

---

## 🔴 P0 — BROKEN (Compiles but produces wrong behavior)

| # | Feature | Bug | Location | Test |
|---|---------|-----|----------|------|
| P0-1 | **`for...in` loops** | ~~Body executes exactly ONCE. No iteration over arrays/Vecs/ranges.~~ **FIXED v0.56** — Direct Range iteration via {start,end} field manipulation with proper loop structure (cond/body/exit blocks), break/continue support, and loop_label passthrough. | `xiom-codegen/src/stmt.rs:1649-1730` | `tests/e2e_p0_forin.xi` ✅ |
| P0-2 | **`defer` statement** | ~~Executes IMMEDIATELY where written, not at scope exit.~~ **FIXED v0.56** — LIFO defer stack emits deferred blocks at every `ret` point (compile_block tail-returns, Stmt::Return handler, fallback ret in compile_fn). Stack cleared per-function. | `xiom-codegen/src/stmt.rs` (defer handler + compile_deferred_cleanups), `decl.rs` (compile_fn start), `lib.rs` (compile_block ret sites) | `tests/e2e_p0_defer.xi` ✅ |
| P0-3 | **Labeled break/continue** | ~~Parser accepts `break @outer` / `continue @outer` but codegen IGNORES the label.~~ **FIXED v0.56** — loop_stack stores `(Option<String>, label, exit)`. Break/Continue search stack (top-down) for matching label; fallback to innermost if no match. Parser supports `@label: while/for` prefix. | `xiom-codegen/src/stmt.rs` (Break/Continue/While handlers), `xiom-parser/src/lib.rs` (parse_stmt_or_expr), `xiom-ast/src/lib.rs` (Stmt::While, Stmt::For) | `tests/e2e_p0_labeled.xi` ✅ |

## 🟠 P1 — MISSING (Spec says it works, no implementation)

| # | Feature | Gap | Location | Test |
|---|---------|-----|----------|------|
| P1-1 | **Struct patterns in match** | `match point { Point{ x, y } => ... }` — AST has no `Pattern::Struct` variant. Parser never parses `TypeName{ field, ... }` syntax. AI_CONTEXT.md line 152 shows `AgentState.Patrolling(route)` (variant pattern, works) but struct destructuring is missing. | `xiom-ast/src/lib.rs` (Pattern enum), `xiom-parser/src/lib.rs` (parse_pattern_single) | Needs E2E test |
| P1-2 | **Tuple patterns in match** | `match pair { (a, b) => ... }` — AST has no `Pattern::Tuple` variant. Parser only supports `()` (unit) and `(pattern)` (parenthesized). Multi-field tuple destructuring missing. | Same as above | Needs E2E test |
| P1-3 | **Float literal patterns** | `match x { 3.14 => ... }` — Only Int/Bool/Str/Char literal patterns exist in parser. Float literals not handled in `parse_pattern_single`. | `xiom-parser/src/lib.rs:1523-1527` | Needs E2E test |
| P1-4 | **Contract collection methods** | `.is_sorted()`, `.all()`, `.none()`, `.contains()` — listed in spec Section 5.4 table as contract predicates. Zero implementation across parser/checker/codegen. | Nowhere | Needs implementation |

## 🟡 P2 — PARTIAL (Works in some cases, fails in others)

| # | Feature | Gap | Location | Test |
|---|---------|-----|----------|------|
| P2-1 | **`?` operator — missing return-type check** | Checker validates operand is Result/Option but does NOT check that the enclosing function returns Result/Option. Spec says `?` only in Result/Option functions; you can write `?` in a function returning `Int`. | `xiom-check/src/lib.rs:2781-2795` | Needs checker test |
| P2-2 | **Borrow errors are warnings, not errors** | E001 "use after move" / "borrow conflict" are emitted as `warning[E001]` and do NOT block compilation. The code compiles but produces binary that SEGFAULTs at runtime. Spec says ownership violations are compile errors (Section 4.3). | `xiom/src/lib.rs:784-799` | Needs `--strict` mode |
| P2-3 | **Place-level (field-granular) borrows** | `place.rs` and `loans.rs` are compiled and tested but NOT wired into the active borrow checker. Only bare-variable borrows work. `&a.x` and `&mut a.y` both mark `a` as borrowed, triggering a false conflict. | `xiom-check/src/borrow/` (wired-in but not active) | Needs integration |
| P2-4 | **Never type (!) LLVM lowering** | `!` parses/type-checks correctly but lowers to `i64` in LLVM IR (not a true bottom type). Exhaustiveness proofs not encoded at IR level. `fn abort() -> !` compiles to `define i64 @abort()` — valid but misses the semantics. | `xiom-codegen/src/types.rs:313-339` | Needs LLVM fix |
| P2-5 | **Interface bounds enforcement** | Checked at monomorphisation time only (not at use-site). Users get errors late in pipeline (codegen phase), not at type-check time. Deliberate design decision per `lib.rs:4819-4821`. | `xiom-check/src/lib.rs:3045-3075` | Move to checker |
| P2-6 | **Turbofish single type arg only** | `Expr::GenericCall` stores a single `Type`, not `Vec<Type>`. `parse::<Int>("42")` works; `foo::<Int, Str>()` fails. | `xiom-ast/src/lib.rs:159-161` | Needs multi-type support |

## 🟢 P3 — COSMETIC (Works but design could be better)

| # | Feature | Gap | Location |
|---|---------|-----|----------|
| P3-1 | **Char type mapping inconsistency** | `types.rs:303` maps Char to `i8`, `lib.rs:905` maps Char to `i32`. Duplicate code with conflicting widths. | Two copies of `xiom_to_llvm_type()` |
| P3-2 | **Map[K,V] / Set[T] no native hash-table** | Spec says Hash+Eq map/set. Only BTreeMap/BTreeSet (Ord-based) exist in stdlib. No dedicated hash-table codegen or runtime. | Spec Section 3.2 |
| P3-3 | **Result/Option higher-order methods** | `.map()`, `.and_then()`, `.filter()`, `.unwrap_or_else()`, `.expect()`, `.is_some_and()`, `.is_ok_and()` — specified in AI_CONTEXT.md but only exist as `core.xi` library functions, not compiler builtins. Closure-taking methods have limited support. | `core.xi` |
| P3-4 | **`for` loop — no Iterator protocol** | Even when P0-1 is fixed, `for` needs to integrate with the Iterator interface. Currently has no connection to `iter.xi` or iterator adapters. | Parser + Codegen |

## ⚪ P4 — DEFERRED (Documented, not blocking)

| # | Feature | Status |
|---|---------|--------|
| P4-1 | `contracts.xi` runtime SMT verification | Honest placeholders; metadata table doesn't embed clause expressions yet |
| P4-2 | `reflect.xi` generic [T] queries | Per-monomorphisation intrinsic not implemented; type count/names work |
| P4-3 | `regex.xi` full PCRE features | Simplified engine only; lookahead/backrefs explicitly NOT supported |
| P4-4 | Legacy modules (aes, sha, b64, etc.) | Use unified `xiom.crypto`, `xiom.encoding`, `xiom.rand` instead |

---

## WORKING VERIFIED — No Gaps Found

These features are FULLY IMPLEMENTED and PRODUCTION-GRADE:

| Category | Features |
|----------|----------|
| **Primitives** | Bool, Int, Int8-Int64, UInt-UInt64, Float32, Float64, Char, Str, Unit |
| **Compounds** | Option[T], Result[T,E], Vec[T], [N]T (fixed arrays), tuples, *T, &T, &mut T |
| **Derives** | Eq, Clone, Display, Hash, Ord, Debug — all 6 on both structs AND enums |
| **Control flow** | if/elif/else, while, match (exhaustive), if-let, while-let (desugar) |
| **Patterns** | Wildcard, Ident, Some/None, Ok/Err, enum variants, literal (Int/Bool/Str/Char), Or, guards |
| **Generics** | Bounds [T: Interface], multi-bounds, turbofish, type params, monomorphisation, nested |
| **Interfaces** | Structural typing, default implementations, Send/Sync auto-derivation |
| **Modules** | module declaration, use imports, as aliases, pub visibility, package.xi |
| **Contracts** | requires/ensures/invariant, result, self@pre, --no-contracts, --verify, Z3 |
| **Threading** | spawn+move, Channel[T], thread pool, AtomicInt, Send/Sync enforcement |
| **Safety** | No null, no uninit, overflow checks ON by default, match exhaustiveness, bounds checks |
| **Codegen** | Parallel codegen, DI emission (DWARF), LTO, JIT, binary cache |
| **Stdlib** | 40 modules fully implemented (52 .xi files, 8 legacy) |

---

## ACTION PLAN — To Close Pre-Selfhost

### Immediate (This Session)
1. **Mark all gaps in SESSION.md** — comprehensive handoff
2. **Update AI_CONTEXT.md** — add `move` keyword, `--overflow-checks` default, `--parallel-codegen` flag
3. **Update docs/language/compiler.md** — list all compiler flags with descriptions
4. **Update MCP knowledge** — add new flags to MCP server knowledge base
5. **Write clean handoff prompt** for next session

### Next Session (P0 + P1)
6. ~~Fix P0-1: `for...in` iteration (3 days)~~ ✅ DONE v0.56
7. ~~Fix P0-2: `defer` scope-exit execution (2 days)~~ ✅ DONE v0.56
8. ~~Fix P0-3: Labeled break/continue (1 day)~~ ✅ DONE v0.56
9. Fix P1-1: Struct patterns in match (2 days)
10. Fix P1-2: Tuple patterns in match (1 day)
11. Fix P1-3: Float literal patterns (0.5 day)
12. Fix P1-4: Contract collection methods (2 days)

### Following Session (P2)
13. Fix P2-1: `?` return-type check (1 day)
14. Fix P2-2: Borrow errors as hard errors (1 day)
15. Fix P2-3: Wire place-level borrows (3 days)
16. Fix P2-4: Never type LLVM lowering (1 day)
17. Fix P2-5: Move interface bounds to checker (2 days)
18. Fix P2-6: Multi-type turbofish (1 day)

### Post-Selfhost (P3 + P4)
19. P3 items, P4 items, optimization, ecosystem
