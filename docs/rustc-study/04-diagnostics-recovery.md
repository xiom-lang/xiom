<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# 04 -- Diagnostics and Error Recovery: rustc vs XIOM

Study-only note (no code changes). Purpose: extract rustc's exact
cascade-prevention contract and compare it with XIOM's checker so that
the REL-20261010-1900 family (bogus W004 unreachable-arm warnings before
the real T001 during unresolved-module recovery) and the C-ORBIT-06
phantom-import family can be fixed from a verified pattern, not from
memory.

Ground truth used:
- rustc clone `E:\repos\rust`, commit `7211ea594cbf137ef6118b1752b959dea7cb9088`
  (read-only; `compiler/` materialized, `tests/ui` read lazily via
  `git ls-tree` / `git show`).
- XIOM worktree `study-rustc-diagnostics-bbb742cf7f39c648`, HEAD
  `495d6672`; all XIOM line citations are to that tree.
- docs: RUSTC-REFERENCE.md (main-worktree copy; rule 1), JSON_
  DIAGNOSTICS_V1.md, COMPILER_READINESS_PLAN.md, COMPILER_BUGS.md.

Stage tags in section 1 mark urgency, not file ownership: S6 = current
diagnostics-quality/polish wave; S7 = selfhost gate; S8-9 = post-selfhost.

---

## 1. Planning input

- [S6] Gate the whole Stage-6 lint wave (W002/W003/W004) on a
  body-local error taint, not on `checking_catalog`. rustc's analogue is
  `TypeckResults::tainted_by_errors` + the `thir_body` bail
  (`rustc_mir_build/src/thir/cx/mod.rs:30-34`); XIOM has a per-checker
  `error_count` (crates/xiom-check/src/lib.rs:235-237, 1896-1898) but no
  per-body flag, and the W004 call sites read only
  `!self.checking_catalog` (lib.rs:6676, 9505).
- [S6] Make the W004 call sites scrutinee-aware: skip when the scrutinee
  checked to `CheckedType::Error` and treat "variant set unknown" as
  distinct from "variant set empty". Today `match_variant_names`
  (lib.rs:9536-9560) returns an empty vector for a poisoned or
  unresolved scrutinee, and `pattern_is_catch_all_v` (lib.rs:5897-5911)
  then classifies a bare variant arm as a binding catch-all -- this is
  the verified W004 cascade mechanism (section 3.3).
- [S6] Give `CheckedType::Error` (crates/xiom-check/src/types.rs:222-223)
  the rustc shape: `TyKind::Error(ErrorGuaranteed)` carries the proof
  (rustc_middle/src/ty/sty.rs:524-527), so a dependent check can ask
  "was this poisoned?" without a parallel lookup. XIOM's poison type is a
  unit variant; the token lives only on `CheckError`/`Expr::Error`.
- [S7] Single minting point for `xiom_ast::ErrorGuaranteed`:
  `ErrorGuaranteed::new()` is `pub` (crates/xiom-ast/src/lib.rs:277-289)
  while rustc's token has a private field and a deprecated
  `unchecked_error_guaranteed` whose only legal caller is
  `DiagCtxtInner::emit_diagnostic`
  (rustc_span/src/lib.rs:2851-2869; rustc_errors/src/lib.rs:1362-1365).
  Any XIOM pass can mint a proof without emitting anything, so the
  "guarantee" is not yet a guarantee.
- [S7] Phantom imports need one error + a placeholder binding. rustc
  reports unresolved imports once, batched per use tree, then plants a
  dummy `Res::Err` binding so later stages never see a hole
  (rustc_resolve/src/imports.rs:729-762, 923-1010). XIOM's `process_use`
  silently `return`s when the module cannot be loaded
  (crates/xiom-check/src/lib.rs:4527-4529, 4614, 4657-4658), so an
  unresolved import produces zero import-site diagnostics and everything
  downstream guesses (C-ORBIT-06).
- [S6] Ordering fix falls out of gating: the driver prints the warning
  stream before the error stream on the failure path
  (crates/xiom/src/lib.rs:1010-1027 before :1028-1078; JSON puts
  warnings first, :1029-1054). Suppressing bogus warnings at the source
  removes the misleading "warning before error" read without changing
  schema v1 (docs/JSON_DIAGNOSTICS_V1.md).
- [S6] Adopt rustc's dedup discipline for the warning stream: a stable
  (code, parentless-span, message) hash set
  (rustc_errors/src/lib.rs:1299-1320, dedup_hash.rs:1-63) instead of the
  per-warning linear scan in `warn_coded_at` (lib.rs:1860-1864).
- [S8-9] Keep error-poisoned bodies out of the m252 body-check cache: the
  cache is documented positive-only (checkcache.rs:22-26), but rustc
  enforces the stronger contract that tokens can never be serialized
  (rustc_span/src/lib.rs:2872-2888). Add an explicit "tainted bodies are
  never recorded" lock test.
- [S8-9] Consider a XIOM analogue of `Level::DelayedBug`
  (rustc_errors/src/lib.rs:1597-1601): "expect another error on this
  path; if none occurs, this was a compiler bug". XIOM's defensive
  recovery fallbacks are silent today; a delayed diagnostic would
  surface real bugs without failing error-free compiles.

Concrete design sketch for error-token gating in `crates/xiom-check`
(target: make W004 cascades impossible; minimal diff, no CheckedType
shape change in slice 1):

```text
Checker fields (lib.rs struct around :235):
    body_tainted: Option<xiom_ast::ErrorGuaranteed>   // per body
    fn taint_body(&mut self); fn body_is_tainted(&self) -> bool

error_with_cause (:1778): after pushing the diagnostic:
    self.taint_body();   // every hard error taints the current body

check_all_bodies (:2112): reset self.body_tainted = None before each
    check_top_decl(item); catalog-prefix logic unchanged.

W004 call sites (Stmt::Match :6673-6679, Expr::Match :9502-9508):
    if !self.checking_catalog && scr_ty != CheckedType::Error
       && !self.body_is_tainted()
       && let Some(variants) = self.match_variant_names_opt(&scr_ty) {
        self.lint_w004_unreachable_arms(arms, &variants);
    }

match_variant_names_opt: Option<Vec<String>>; None = "scrutinee
    shape unknown" (poisoned, wildcard-only, or Named with no
    registered enum parents and no builtin match). Today's empty-vec
    collapse must split into: Some(empty) = genuine non-enum;
    None = unknown, the lint must not guess.

Slice 2 (follow-up): CheckedType::Error(ErrorGuaranteed) or a
    parallel poisoned-type set, mirroring TyKind::Error(guar).
```

Rationale: rustc's gate is local (per body/query), not global: a match
in a clean function still lints even if another function errored, while
a match whose scrutinee failed typeck never reaches the analysis because
`thir_body` returns `Err(ErrorGuaranteed)` first. The sketch reproduces
that locality.

---

## 2. How rustc does it

### 2.1 The token that cannot be forged

`rustc_span/src/lib.rs:2851-2869` -- doc: "Useful type to use with
`Result<>` indicate that an error has already been reported to the
user, so no need to continue checking"; "The `()` field is necessary:
it is non-`pub`, which means values of this type cannot be constructed
outside of this crate"; `unchecked_error_guaranteed()` is
`#[deprecated = "should only be used in `DiagCtxtInner::emit_diagnostic`"]`;
`raise_fatal()` turns the token into a hard abort when the caller
cannot proceed.

Serialization panics on purpose (`:2872-2888`): an `ErrorGuaranteed` can
never enter metadata or an incremental cache, so a cached artifact can
never be built from error-poisoned data.

### 2.2 Minting and the DiagCtxt contract

`rustc_errors/src/lib.rs`:

- `emit_diagnostic` "Return value is only `Some` if the level is `Error`
  or `DelayedBug`" (`:1202-1207`); Warnings/notes return `None`
  (`:1260-1290`).
- The only mint site (`:1362-1365`): "This `unchecked_error_guaranteed`
  is valid. It is where the `ErrorGuaranteed` for errors and lint errors
  originates." Errors land in `err_guars`, lint errors in
  `lint_err_guars` (`:1367-1374`); the taint cell is set here too
  (`:1370-1373`).
- `has_errors() -> Option<ErrorGuaranteed>` (`:747-762`); `abort_if_errors`
  (`:851`); `print_error_count` prints "aborting due to N previous
  errors" (`:764-798`).
- `DiagCtxtHandle::err/span_err/emit_err` return `ErrorGuaranteed`
  (`:1036-1067`); `bug`/`span_bug` panic (ICE) (`:969-983`);
  `delayed_bug`/`span_delayed_bug` (`:1069-1086`).
- `Level::DelayedBug` (`:1597-1601`): "lets you register an error without
  emitting it... If compilation ends without any other errors occurring,
  this will be emitted as a bug. Otherwise, it will be silently
  dropped." Once a real error exists, delayed bugs are discarded
  (`:1240-1257`, `:1351-1360`) -- the "expect other errors" channel can
  never produce user-visible cascade noise.

### 2.3 The gating pattern: taint cells and body-local bailouts

The pattern that makes cascading diagnostics structurally impossible:

1. A context that owns a body-sized worker declares a
   `Cell<Option<ErrorGuaranteed>>` taint cell and wraps its diagnostic
   context: `DiagCtxt::into_taintable(&cell)` -- "emitting errors will
   automatically set the `Option<ErrorGuaranteed>`" without manual work
   at every emission site (`rustc_errors/src/lib.rs:563-571`; set path
   `:1367-1373`).
2. The body's result type stores the taint:
   `TypeckResults::tainted_by_errors: Option<ErrorGuaranteed>` --
   "If any errors occurred while type-checking this body, this field
   will be set" (`rustc_middle/src/ty/typeck_results.rs:165-167`).
3. Every dependent query checks it and bails with the SAME token before
   doing any work: `check_tainted_by_errors` returns `Err(guar)`
   (`rustc_hir_analysis/src/collect.rs:281-286`); `type_of` substitutes
   `Ty::new_error(tcx, e)` when "tainted && !output.references_error()"
   (`collect/type_of.rs:281-287`); borrowck's `finalize` returns
   `Err(guar)` if tainted (`rustc_borrowck/src/root_cx.rs:74-98`).
4. Consumer queries propagate with `?` and never emit their own
   diagnostics on a tainted path: MIR building bails to
   `construct_error` on a failed `check_match`/`thir_body`
   (`rustc_mir_build/src/builder/mod.rs:67-79`).

Because the token flows through the return type, the "skip dependent
work" decision is enforced by the type system; a new check physically
cannot run without deciding what to do with the taint.

### 2.4 The exact analogue of XIOM's W004: match checking

- `check_match` (query registered at `rustc_mir_build/src/lib.rs:20`)
  starts with (`thir/pattern/check_match.rs:30-32`):

  `let typeck_results = tcx.typeck(def_id);`
  `let (thir, expr) = tcx.thir_body(def_id)?;`

  The `?` is the cascade gate: if typeck of this body failed,
  `thir_body` returns `Err(ErrorGuaranteed)` and the match/usefulness
  analysis never runs -- `unreachable_patterns` warnings are IMPOSSIBLE
  for a body whose scrutinee types failed.
- `thir_body` itself holds the check
  (`rustc_mir_build/src/thir/cx/mod.rs:30-34`):

  `if let Some(reported) = cx.typeck_results.tainted_by_errors {`
  `    return Err(reported);`
  `}`

- Only after a successful analysis are arm usefulness results walked and
  `report_unreachable_pattern` called (`check_match.rs:396-422`); the
  lint is `UNREACHABLE_PATTERNS` (`:962-1035`). The query returns
  `Result<(), ErrorGuaranteed>` (`:30`), so the token keeps flowing.
- Variant/binding ambiguity is handled by E0170 + a `Foo::Bar`
  suggestion, never by guessing reachability
  (`tests/ui/match/unreachable-pattern-if-variant-not-imported-19100.*`):
  the analysis knows the variant list because lowering succeeded; when
  it cannot know, it does not run.

### 2.5 Unresolved imports: one error, batched, then a dummy binding

`rustc_resolve/src/imports.rs:923-1010` (`finalize_imports`):

- All unresolved imports are collected, de-duplicated by span
  (`:968-972`), coalesced per `use` line/tree, and reported via
  `throw_unresolved_import_error` (`:965, :979, :1008`) -- so one broken
  use tree is ONE E0432.
- Immediately after classification, EACH unresolved import gets
  `import_dummy_binding` (`:945-947`): "create a dummy import resolution
  for it so that later resolve stages won't complain" -- a `Res::Err`
  placeholder decl is planted and marked used "to avoid duplicate
  diagnostics" (`:729-762`).
- `throw_unresolved_import_error`
  (`rustc_resolve/src/diagnostics/impls.rs:140-160`): skips import
  errors for modules that already had unrecovered parse errors
  ("Skip `use` errors ... if `foo.rs` has unrecovered parse errors"),
  combines multiple unresolved paths into one diagnostic (`:162-176`),
  and `delayed_bug`s when called with nothing to report (`:157-159`) --
  recovery paths assert rather than emit noise.
- Golden test: `tests/ui/unresolved/unresolved-import-recovery.rs` is
  explicitly "Check that unresolved imports do not create additional
  errors and ICEs": after the E0432, `m::unresolved` used in three match
  patterns produces ZERO further diagnostics because the dummy binding
  satisfies name resolution.

### 2.6 Unresolved types: error types carry their proof

- HIR type lowering returns `Result<_, ErrorGuaranteed>` throughout
  (e.g. `ResolvedStructPath.res: Result<Res, ErrorGuaranteed>`,
  `rustc_hir_analysis/src/hir_ty_lowering/mod.rs:129-132`); the
  `TyKind::Error(guar)` variant is constructed only together with an
  emitted diagnostic: `Ty::new_error(tcx, guar)`
  (`rustc_middle/src/ty/sty.rs:524-527`), and
  `new_error_with_message` registers a `span_delayed_bug` safety net
  (`:529-545`). Consumers branch on `ty.references_error()`
  (`rustc_type_ir/src/visit.rs:283`) or on the propagated `Result` --
  never on sentinel strings.

### 2.7 Suggestion engine basics

- Suggestions are structured data attached to a `Diag`, not ad-hoc
  message text: `span_suggestion` / `span_suggestion_with_style` /
  `span_suggestion_verbose` build a `CodeSuggestion` with one
  `Substitution` per span replacement
  (`rustc_errors/src/diagnostic.rs:840-913`; struct at
  `rustc_errors/src/lib.rs:155-200`).
- Every suggestion carries `Applicability` (`MachineApplicable`,
  `MaybeIncorrect`, `HasPlaceholders`, `Unspecified`;
  `rustc_lint_defs/src/lib.rs:55-82`); tools apply only the first class
  and `run-rustfix` + `.fixed` tests lock the round-trip.
- Style conventions on `span_suggestion` (`diagnostic.rs:845-852`): no
  trailing punctuation, no "did you mean", imperative phrasing.

### 2.8 UI-test golden discipline

- Tests are `.rs` inputs with inline expectations (`//~ ERROR ...`,
  `//~^`, `//~|`; compiletest grammar at
  `src/tools/compiletest/src/errors.rs:85-133`) plus an exact `.stderr`
  snapshot with `$DIR` normalization.
- Read examples (fetched lazily with `git show`):
  - `tests/ui/error-codes/E0432.rs` + `.stderr` -- a good
    unresolved-import diagnostic: one error, caret span, one actionable
    `help:` suggestion.
  - `tests/ui/unresolved/unresolved-import-recovery.rs` + `.stderr` --
    cascade-freedom encoded as a golden test.
  - `tests/ui/match/unreachable-pattern-if-variant-not-imported-19100.rs`
    + `.stderr` + `.fixed` -- variant/binding ambiguity resolved by a
    coded diagnostic with a machine-applicable suggestion.

### 2.9 Dedup and presentation

- `emit_diagnostic` hashes every diagnostic and skips duplicates
  (`emitted_diagnostics: FxHashSet<Hash128>`,
  `rustc_errors/src/lib.rs:1299-1320`); the hash ignores span parents so
  incremental and non-incremental runs behave the same
  (`rustc_errors/src/dedup_hash.rs:1-63`). Duplicates feed the final
  "N warnings / N errors" summary (`:764-798`).

---

## 3. How XIOM does it today

### 3.1 JSON envelope contract (verified)

`docs/JSON_DIAGNOSTICS_V1.md` (v1, round 45):

- One envelope object `{schema_version: 1, diagnostics: [...]}`, oldest
  first; warning `code` covers W000-class checker warnings and the
  Stage-6 lints W002-W008 (docs/JSON_DIAGNOSTICS_V1.md:36-46).
- W004 is "unreachable match arm" (`:39`); W005 is codegen-only
  (`:52-56`). Serializer `xiom::diagnostics_json` + `Diagnostic` struct
  (crates/xiom/src/lib.rs:193). Check-only success is a status object
  (`:27-32`).

### 3.2 W004 sites (all verified in this worktree)

- Lint body: `lint_w004_unreachable_arms`
  (crates/xiom-check/src/lib.rs:5913-5955). Rules: an arm is unreachable
  when (a) an earlier UNGUARDED catch-all shadows it, or (b) its
  structural shadow key (`pattern_shadow_key`, :5962-6001) was already
  seen unguarded.
- Catch-all classification: `pattern_is_catch_all_v` (:5897-5911) --
  a bare `Ident` is a variant only when it names one of `variants`;
  otherwise it is a binding (catch-all).
- Call sites: `Stmt::Match` (:6673-6679) and `Expr::Match`
  (:9502-9508). Both guard ONLY with `if !self.checking_catalog`.
- Variant knowledge: `match_variant_names` (:9536-9560) returns
  `Vec::new()` for any scrutinee that is not a `Named` type after alias
  resolution -- including `CheckedType::Error` -- and for a `Named` type
  with no registered enum parents.
- Emission: `warn_coded_at` (:1850-1883) dedups identical
  (code, span, message) warnings by linear scan of `self.warnings`
  (:1860-1864), prefixes catalog findings, and flips to hard errors only
  under `strict_catalog_findings`.

### 3.3 The cascade mechanism (REL-20261010-1900 root shape)

Verified chain, each link read in source:

1. A match arms over a scrutinee whose enum surface never resolved
   (phantom/unresolved import, undeclared signature type, or an earlier
   body error that poisoned the value expression).
2. `check_expr` returns `CheckedType::Error` for the poisoned value, or
   a `Named` type whose enum never registered.
3. `match_variant_names` returns `[]` (lib.rs:9536-9540 return early for
   non-Named; no enumeration possible for an unregistered Named).
4. In `lint_w004_unreachable_arms`, the first bare `Ident` arm is
   classified by `pattern_is_catch_all_v` as a catch-all binding
   (`!variants.contains(leaf)` == true), setting
   `shadowed_by_catch_all = true` (:5943-5945).
5. Every subsequent arm is then emitted as W004 (:5934-5940) -- plural
   bogus warnings.
6. The real T001 arrives later in the same pass / driver stage (e.g. the
   "foreign structural twin" mismatch of C-ORBIT-06), and the driver
   prints warnings BEFORE errors (crates/xiom/src/lib.rs:1010-1027 vs
   :1028-1078; JSON: warnings ahead of errors, :1029-1054).

Note: `check_program` (lib.rs:2145-2183) appends UNCODED warnings to the
error list when errors exist (:2166-2176) but keeps coded warnings
(W002-W008) in the warning stream -- W004 survives to the output even
when the compile already fails.

### 3.4 Recovery sites (verified)

- Parse/lex gate: the driver returns before type checking when any
  parse_error/lex_error exists (crates/xiom/src/lib.rs:570-573). This is
  the one existing whole-pipeline cascade guard.
- Poison propagation: `error_with_cause` mints
  `xiom_ast::ErrorGuaranteed::new()`, pushes a `CheckError` with
  `guaranteed`, increments `error_count`, returns `CheckedType::Error`
  (lib.rs:1778-1803). Many consumers honor it (`!= CheckedType::Error`
  checks at :6632, :6642, :6658, :6700; `Expr::Error` returns Error
  silently at :9486). The W004 lint does not.
- Phantom imports: `process_use` (lib.rs:4444) silently returns when the
  catalog cannot load the module (:4527-4529) or an intermediate
  submodule walk dead-ends (:4614, :4657-4658). No import-site error is
  emitted; xiom-graph has `GraphError::ModuleNotFound`
  (crates/xiom-graph/src/lib.rs:41-45, 59-67) but the check-only path
  does not require the graph.
- Unresolved type names at use sites produce "unknown type '...'" in
  some positions (:9253) but `Expr::Struct` still falls back to
  `CheckedType::named(name)` (:9269), and `from_ast_type`-style paths can
  yield `Named("_")`, so "surface unknown" and "genuine type" are not
  distinguishable downstream.
- Token weakness: `xiom_ast::ErrorGuaranteed` is a `pub struct` with a
  `pub fn new()` (crates/xiom-ast/src/lib.rs:273-289) -- any crate can
  mint proofs; the parser does exactly that straight into `ParseError`
  (crates/xiom-parser/src/lib.rs:101, 104, 343, 359; field at :2707-2714).
  `Expr::Error(ErrorGuaranteed, Span)` exists (:261-264) and consumers
  treat it as poison (check :9486, codegen skips, type_qualify skips),
  but no pass constructs that node today.
- Poison type shape: `CheckedType::Error` is a unit variant
  (crates/xiom-check/src/types.rs:222-223); proof tokens live on
  `CheckError.guaranteed` (types.rs:610-623) and `Expr::Error`.
- Catalog-body cache: positive-only by documentation and guards
  (checkcache.rs:22-26); "only bodies with ZERO new diagnostics are
  recorded".
- Tests locking current W004 behavior: `m154_w004_unreachable_warns`,
  `m154_w004_guarded_stays_silent`, `m220_w004_bare_variants_silent`
  (crates/xiom/tests/checker_locks.rs:239-279). No lock asserts
  "error suppresses W004".

---

## 4. Gap table

| Dimension | rustc (path) | XIOM today (path) | Consequence | Adopt |
|---|---|---|---|---|
| Proof token forgery | private field; deprecated unchecked mint (rustc_span:2851-2869) | `pub fn new()` (xiom-ast:277-289) | "guarantee" is advisory; a pass can mint without emitting | make minting one emitter method (S7) |
| Poison carries proof | `TyKind::Error(guar)` (ty/sty:524-527) | `CheckedType::Error` unit (types:222-223) | cannot ask "was this poisoned?" structurally | attach token or parallel poison set (S6 slice 2) |
| Per-body taint | `tainted_by_errors` (typeck_results:165-167) | none (global `error_count` only, lib.rs:235-237) | checks in an errored body keep running | add `body_tainted` (S6) |
| Dependent lint gate | `thir_body` bails `Err(guar)`; `check_match` `?` (thir/cx:30-34; check_match:30-32) | W004 runs unconditionally unless catalog (lib.rs:6676, 9505) | bogus W004 cascade (REL-20261010-1900) | gate on taint + scrutinee poison (S6) |
| Variant knowledge | analysis only after successful lowering; E0170 for ambiguity (19100 test) | empty variant vec means both "non-enum" and "unknown" (lib.rs:9536) | bare arms treated as catch-alls | tri-state variant knowledge (S6) |
| Unresolved import | one batched E0432 + dummy binding (imports.rs:729-762, 923-1010) | silent `return`, zero diagnostics (lib.rs:4527-4658) | phantom imports silently accepted (C-ORBIT-06) | one error + dummy binding (S7) |
| Parse-error suppression | skip import errors for parse-broken modules (resolve/impls:150-156) | parse gate exists at driver level (xiom lib.rs:570-573) | OK at pipeline level; per-module refinement missing | mirror the per-module skip (S7) |
| Diagnostic dedup | global stable-hash set (errors/lib.rs:1299-1320; dedup_hash.rs) | linear scan per coded warning (lib.rs:1860-1864) | N^2 latency on lint-heavy files | hash set (S6) |
| Delayed-bug channel | `Level::DelayedBug` dropped once real errors exist (errors:1597-1601, 1240-1257) | none; defensive fallbacks silent | internal bugs during recovery stay invisible | add delayed channel (S8-9) |
| Cache discipline | token cannot be serialized (rustc_span:2872-2888) | positive-only cache, no taint test (checkcache:22-26) | drift risk if guards loosen | lock test (S8-9) |
| Suggestion shape | structured substitutions + applicability (diagnostic.rs:840-913; lint_defs:55-82) | single free-text `suggestion` string (JSON v1:44) | no machine-applicable fixes in JSON consumers | extend v1 with optional fields only (S8-9) |
| Presentation | one stream, `aborting due to N previous errors` (errors:764-798) | warnings printed before errors (xiom lib.rs:1010-1078) | misleading ordering during recovery | gating makes this moot (S6) |

---

## 5. Optimization & performance opportunities

Diagnostic-side only; all measured against the current code shapes.

- Lint dedup cost. `warn_coded_at` scans every accumulated warning for
  every emission candidate (lib.rs:1860-1864): O(W^2) on lint-heavy
  files. rustc uses a global stable-hash set (errors/lib.rs:1299-1320);
  a per-run `HashSet<(code_hash, span_lo, span_hi, message_hash)>`
  removes the quadratic term.
- Shadow-key allocation. `pattern_shadow_key` builds a `String` per arm
  (lib.rs:5962-6001) and `seen` is a `Vec<String>`; an `FxHashSet` of
  borrowed keys cuts per-match allocation (constant-factor, low
  priority).
- Taint check is free. `Option<ErrorGuaranteed>` is a nullable
  ZST-sized copy; a per-body reset in `check_all_bodies`
  (lib.rs:2112-2116) costs one store per top decl and removes
  potentially unbounded lint work on error-heavy inputs (N poisoned
  matches today pay N lint walks).
- Fewer diagnostics, faster CI. Suppressed cascades shrink the JSON
  payload, stderr rendering, LSP mapping, and MCP payloads; the
  failure path currently pays a warn-then-error double pass
  (xiom lib.rs:1010-1078).
- Cache hygiene as latency. The m252 body cache is the biggest warm-path
  win (checkcache.rs:1-26); an explicit taint guard keeps it
  positive-only (rustc's refusal to serialize tokens is the reference
  policy).
- Diagnostic latency visibility. `XIOM_TIMINGS` instruments check phases
  (crates/xiom-check/src/lib.rs:2134-2143); counting emitted vs
  suppressed lints would let the campaign prove the gating effect on
  real corpora (rustc ships a comparable precedent: timings.rs:39-60).

---

## 6. Adoption proposals

Ordered, each PR-sized, all in `crates/xiom-check` unless noted.

1. `lib.rs` -- add `body_tainted: Option<xiom_ast::ErrorGuaranteed>` to
   `Checker` (struct around :235); set it in `error_with_cause` (:1778)
   and in the two strict-catalog hard-error paths of
   `warn_at`/`warn_coded_at` (:1826-1835, :1865-1875); reset per top
   decl in `check_all_bodies` (:2112); expose `fn body_is_tainted`.
2. `lib.rs` -- add `match_variant_names_opt -> Option<Vec<String>>`
   next to `match_variant_names` (:9536): `None` for a poisoned
   scrutinee, a non-`Named` scrutinee after alias resolution, or a
   `Named` type with zero registered enum parents (excluding the
   synthetic wildcard/`Tuple*` leniency forms). Builtins
   (Option/Result/Bool) keep `Some(vec![...])`.
3. `lib.rs` -- at both W004 call sites (:6676-6679, :9505-9508) guard
   `!self.checking_catalog && scr_ty != CheckedType::Error &&
   !self.body_is_tainted()`, then lint only with `Some(variants)`.
   This is the REL-20261010-1900 fix (one hunk plus helpers).
4. `lib.rs` -- `process_use` (:4444): replace the silent `return`s
   (:4527-4529, :4614, :4657-4658) with a recorded unresolved-import
   entry; after `check_program`, surface ONE T001-class diagnostic per
   failed use tree (batched style of `throw_unresolved_import_error`)
   plus a dummy binding so bare-name lookups resolve to
   `CheckedType::Error` instead of "unknown type" or a same-leaf twin.
   This is the C-ORBIT-06 fix.
5. `checkcache.rs` -- guard/test that tainted bodies are never recorded
   (positive-only doc at :22-26).
6. `crates/xiom/tests/checker_locks.rs` -- add locks mirroring :239-279:
   "hard T001 must not emit W004"; "unresolved import yields exactly
   one import error and no consumer cascade".
7. `crates/xiom-ast/src/lib.rs` -- (breaking, propose separately) make
   `ErrorGuaranteed::new` crate-internal or deprecated in favor of an
   emitter-only constructor (S7 hardening once 1-3 land).
8. `Diagnostic` (crates/xiom/src/lib.rs) -- keep JSON v1; add an
   optional `applicability` field only after a structured-suggestion
   engine exists; no removals (docs/JSON_DIAGNOSTICS_V1.md:48-50).

---

## 7. Open-item relevance

### REL-20261010-1900 (bogus W004 before real T001)

Verified mechanism, fully traced to source (section 3.3): the W004 lint
does not observe either the scrutinee's poison state or the body's error
taint, and `match_variant_names` collapses "unresolved/unknown" to an
empty variant list; `pattern_is_catch_all_v` then reclassifies bare
variant arms as bindings, so the first arm shadows the rest and the
whole remainder of the match is warned unreachable. The warning stream
is rendered ahead of the error stream by design (xiom lib.rs:1010-1078),
which is why the bogus warnings read as "before the real T001".

Fix direction (section 6 proposals 1-3) mirrors rustc exactly:
- body taint gate == `TypeckResults::tainted_by_errors` +
  `thir_body` bail;
- scrutinee poison check == `TyKind::Error(guar)` never reaching
  `check_match`;
- tri-state variant knowledge == "the analysis knows the variant list
  because lowering succeeded; otherwise it does not run".
Expected result: a poisoned or unresolved match produces zero W004; a
clean match in the same file still lints (rustc locality, not a global
mute).

### C-ORBIT-06 (phantom imports / undeclared signature types)

`docs/COMPILER_BUGS.md:14785-14788` documents: "phantom imports and
undeclared signature types are silently accepted; a foreign structural
twin turns it into a hard T001; bogus W004 unreachable arm" with a repro
referenced at `docs/repro/phantom-core-import/` (not present in this
worktree; mechanism verified from source instead). Three independent
observations in XIOM source:

1. `process_use` silent failure -- no import-site diagnostic at all
   (lib.rs:4527-4658). rustc instead reports one batched E0432 and
   installs a dummy binding (imports.rs:729-762, 923-1010).
2. Undeclared signature types are not "unknown" downstream:
   `Expr::Struct` falls back to `CheckedType::named` (lib.rs:9269) and
   same-leaf type qualification (type_qualify.rs:1-52, 221-264) can
   rewrite references to the colliding twin -- the "foreign structural
   twin" path.
3. The bogus W004 is the SAME cascade as REL-20261010-1900 (both are
   the missing taint gate), which is why a single fix serves both items.

Adoption for C-ORBIT-06 is proposals 4 and 2/3: one clear import error,
placeholder bindings that poison consumers, and no lint guessing on a
surface that failed to resolve.

---

## 8. Sources

rustc (commit 7211ea594cbf137ef6118b1752b959dea7cb9088, read-only):

- compiler/rustc_span/src/lib.rs:2851-2888 -- ErrorGuaranteed (private
  field, deprecated mint, serialization panics).
- compiler/rustc_errors/src/lib.rs -- :155-200 CodeSuggestion;
  :563-571 into_taintable; :747-798 has_errors/print_error_count;
  :840-913 suggestions; :851 abort_if_errors; :1036-1086 err APIs;
  :1202-1207 emit contract; :1240-1257 delayed-bug collapse;
  :1299-1320 dedup; :1362-1375 mint + taint; :1585-1601 Level.
- compiler/rustc_errors/src/dedup_hash.rs:1-63;
  compiler/rustc_middle/src/ty/typeck_results.rs:165-167;
  compiler/rustc_middle/src/ty/sty.rs:524-545;
  compiler/rustc_type_ir/src/visit.rs:283.
- compiler/rustc_mir_build/src/thir/cx/mod.rs:30-34;
  thir/pattern/check_match.rs:30-32, 396-422, 962-1035;
  builder/mod.rs:67-79; lib.rs:20.
- compiler/rustc_hir_analysis/src/collect.rs:281-286;
  collect/type_of.rs:281-287; hir_ty_lowering/mod.rs:129-132, 3192-3240;
  compiler/rustc_borrowck/src/root_cx.rs:74-98.
- compiler/rustc_resolve/src/imports.rs:729-762, 923-1010;
  diagnostics/impls.rs:140-160; compiler/rustc_lint_defs/src/lib.rs:55-82.
- tests/ui (via `git show`): error-codes/E0432.rs+.stderr;
  unresolved/unresolved-import-recovery.rs+.stderr;
  match/unreachable-pattern-if-variant-not-imported-19100.rs+.stderr
  (+.fixed); grammar in src/tools/compiletest/src/errors.rs:85-133.

XIOM (worktree HEAD 495d6672):

- docs/JSON_DIAGNOSTICS_V1.md:36-56; docs/COMPILER_BUGS.md:14785-14788;
  docs/COMPILER_READINESS_PLAN.md Stage 1/5/6 notes;
  docs/RUSTC-REFERENCE.md rule 1 + reference map.
- crates/xiom-check/src/lib.rs -- :1778-1803 error_with_cause;
  :1850-1883 warn_coded_at; :2112-2116 check_all_bodies; :2145-2183
  check_program; :2134-2143 XIOM_TIMINGS; :4444-4658 process_use;
  :5667-5739 check_block; :5897-5911 pattern_is_catch_all_v;
  :5913-6001 W004 lint + shadow keys; :6673-6679 and :9502-9508 W004
  call sites; :9250-9270 unknown-type fallbacks; :9536-9560
  match_variant_names.
- crates/xiom-check/src/types.rs:200-226 CheckedType; :604-623
  CheckError; checkcache.rs:1-26 (positive-only cache);
  type_qualify.rs:1-52, 221-264 (same-leaf qualification).
- crates/xiom-ast/src/lib.rs:261-289 Expr::Error + ErrorGuaranteed;
  crates/xiom-parser/src/lib.rs:101, 104, 343, 359, 2707-2714.
- crates/xiom/src/lib.rs -- :193 Diagnostic; :570-573 parse gate;
  :658-670 warnings before error branch; :998-1078 failure path
  (warnings ahead of errors; JSON warnings-first at :1029-1054).
- crates/xiom-graph/src/lib.rs:41-67; xiom/tests/checker_locks.rs:239-279.
