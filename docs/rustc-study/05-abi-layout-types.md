# 05 -- ABI, layout, and type identity: rustc ground truth vs XIOM name strings

Scope: how rustc derives element types, field types, and return modes from
TYPE + LAYOUT alone, and why no name-based re-derivation is possible in its
architecture; what XIOM's codegen does today; and the migration path that
keeps our byte-identical IR goal feasible.

References:
- rustc clone: `E:\repos\rust` at `7211ea594cbf137ef6118b1752b959dea7cb9088`
  (2026-10-11). All rustc citations are path:line in that clone.
- XIOM: this worktree (`study/rustc-abi-layout`, base `495d6672`, 2026-10-10).
  IMPORTANT: this branch PREDATES the m254 (`b443e970`) and m256 (`458d0afb`)
  fixes that are on main; both were read with `git show` from the main
  worktree and are marked as such below.
- Bus items: `E:\xiom-lang\xiom-relays\items\REL-20261010-1904-bindings-2.md`
  (fp returns through fn-pointer cast calls), plus the m250/m254/m256 /
  M19 material in `docs/COMPILER_BUGS.md` (main worktree, lines cited).

Terminology note (verified, not from memory): the old `LayoutS` type no
longer exists in this rustc revision. Its successor is
`LayoutData<FieldIdx, VariantIdx>` (`compiler/rustc_abi/src/lib.rs:2223`)
wrapped by `TyAndLayout<'a, Ty>` (`compiler/rustc_abi/src/layout/ty.rs:76`).
`docs/RUSTC-REFERENCE.md` still says "LayoutS"; this file uses the names that
are actually in the clone.

## 1. Planning input

Tag legend: [S6]/[S7]/[S8-9] name the reference areas of
`docs/RUSTC-REFERENCE.md` (6 TLS, 7 monomorphisation/reachability,
8 IR verification, 9 query/incremental) that a bullet feeds; the Stage N
numbers are `docs/COMPILER_READINESS_PLAN.md`.

- [S7] Land a typed `LocalTypeTable`: binding sites (Stmt::Let/Var, match
  bindings, params) record a STRUCTURED type (container base + element, tuple
  elements, fn-ptr signature), not just a rendered name string. The three
  tuple namers and the fn-ptr call paths then read one record. This is the
  structural precondition for closing REL-20261010-1904 and the m250/m254/m256
  family, not another namer patch.
- [S7] REL-20261010-1904 fix direction (verified against code): record the
  fn-pointer signature at binding (`let f = sym as fn() -> Float64`) and at
  call emission derive `ret` and the LLVM fn type from that record -- the
  analogue of `fn_abi_of_fn_ptr` (rustc_ty_utils/src/abi.rs:291). Today
  `fn_ptr_return_types` is written ONLY for `Type::Fn` parameters
  (decl.rs:1764-1768) and reset per function (decl.rs:1295).
- [S7] Stage 7 differential gate: an fp-returning fn-pointer call must be an
  explicit -O0/-O2/-O3 + clang/gcc fixture. rustc's ABI/layout is
  opt-level-independent (FnAbi comes from `layout_of(Ty)`); any opt-level
  divergence in XIOM is definitionally our bug.
- [S6] Remove string round-trips from emission hot paths:
  `parse_struct_field_types` (lib.rs:8810) prefers registered layout but still
  falls back to splitting `Tuple__Int__Float64` on `_`; tuple naming builds
  names by `join("__")` in three places. One interned type id removes repeated
  allocation and comparison without changing a single IR byte.
- [S6] Memoize layout: rustc computes `layout_of` once per (type, typing env)
  and shares it. XIOM recomputes `infer_llvm_type`/`llvm_type_for` per call
  site. A per-mono-instance layout cache is the cheap Stage 6 win.
- [S8-9] Adopt structural IR verification: rustc runs `layout_sanity_check`
  (rustc_ty_utils/src/layout/invariant.rs:10) and `fn_abi_sanity_check`
  (rustc_ty_utils/src/abi.rs:425) on every computed layout/ABI, and has
  `-Z verify-llvm-ir` (rustc_codegen_ssa/src/back/write.rs:102). XIOM should
  assert, before clang: every emitted `call`'s return/args match the callee
  declaration, and every aggregate store matches the declared field type.
  That check alone would have caught m250/m254/m256 and REL-1904 at build time.
- [S8-9] Shape the typed layer as a cacheable query keyed by the
  monomorphised instance (`Instance { def, args }`, rustc_middle/src/ty/
  instance.rs:23-36), so an incremental cache can never leak one
  instantiation's types into another (the m250 fallout class).
- [S7] Regression locks: for every fix in this family, lock the IR shape
  (lines like `call double %fnptr(` and `%struct.Tuple__Float64__Int`), not
  only the runtime exit code. m254 needed the same patch in three namers
  (decl.rs/expr.rs/lib.rs) precisely because the tests did not assert the
  shared naming contract.

## 2. How rustc does it

### 2.1 Type identity: interned types, instances, no names

- `pub struct Ty<'tcx>(Interned<'tcx, WithCachedTypeInfo<TyKind<'tcx>>>)`
  (rustc_middle/src/ty/mod.rs:517). Types are interned; equality and hashing
  are pointer/structural identity, computed from the `TyKind` tree --
  `Vec<f64>`, `(f64, i64)`, `fn() -> f64` are distinct values with no
  textual spelling anywhere in the pipeline.
- Monomorphisation is on-the-fly and never materializes a "monomorphized MIR
  type name": `Instance { def: InstanceKind<'tcx>, args: GenericArgsRef<'tcx> }`
  (rustc_middle/src/ty/instance.rs:23-36). `InstanceKind` covers `Item`,
  `Intrinsic`, `LlvmIntrinsic`, `Virtual`, `Shim` (instance.rs:62-96).
- The mono collector (`rustc_monomorphize/src/collector.rs:1-110`) walks HIR
  roots and then MIR uses to build the mono item graph. Reachability is a
  property of the graph, never of a symbol string.
- Symbols ARE strings, but they are rendered at the very end from
  `DefPath` + generic args, not used as type input.

### 2.2 Layout is computed, stored per type, and re-verified

- Entry point: `layout_of` query (rustc_ty_utils/src/layout.rs:40), real work
  in `layout_of_uncached` (layout.rs:207). Everything downstream (`codegen`,
  `const_eval`, `FnAbi`) calls this query.
- Result: `TyAndLayout<'a, Ty> { ty: Ty, layout: Layout<'a> }`
  (rustc_abi/src/layout/ty.rs:76), which derefs to
  `LayoutData<FieldIdx, VariantIdx>` (rustc_abi/src/lib.rs:2223):
  - `fields: FieldsShape<FieldIdx>` (offsets/order), `variants` (below),
    `backend_repr: BackendRepr` (Scalar / ScalarPair / Memory / SimdVector /
    SimdScalableVector) -- the ONLY input the ABI reads for pass classification,
  - `largest_niche: Option<Niche>` (lib.rs:2247) and `Niche` with
    `offset/value/valid_range` (lib.rs:2113-2137),
  - `repr_c: bool` (lib.rs:2272), propagated from `repr.c()` and through
    `repr(transparent)` (rustc_abi/src/layout.rs:506-507, :1421).
- Variants are typed per variant:
  `Variants::Multiple { tag: Scalar, tag_encoding, tag_field, variants:
  IndexVec<VariantIdx, VariantLayout<_>> }` (rustc_abi/src/lib.rs:2032-2069);
  `TagEncoding::Niche { untagged_variant, niche_variants, niche_start }`
  (lib.rs:2074-2111) is how `Option<&T>` etc. need no extra tag. A "field"
  is a variant-relative index, not a name:
  `TyAbiInterface::ty_and_layout_for_variant / ty_and_layout_field`
  (rustc_middle/src/ty/layout.rs:804, :848) resolve a field's type from the
  variant it belongs to. There is no global field-name table and therefore no
  M19-style collision to erase.
- `layout_sanity_check` (rustc_ty_utils/src/layout/invariant.rs:10) checks
  size/align consistency, uninhabitedness invariants, and niche/scalar
  invariants after computing every layout.

### 2.3 ABI derived from type + layout

- `ArgAbi { layout: TyAndLayout, mode: PassMode }`
  (rustc_target/src/callconv/mod.rs:394). `PassMode` (mod.rs:59): `Ignore`
  (ZST), `Direct` (Scalar/Vector), `Pair` (ScalarPair), `Cast`, `Indirect`
  (hidden pointer, incl. sret), `IndirectUnsized`.
- `ArgAbi::new` (mod.rs:409-424) maps `layout.backend_repr`:
  Scalar -> Direct, ScalarPair -> Pair, Memory -> Indirect. `f64` is a
  `Scalar` with `Primitive::Float`, so a `fn() -> f64` return is
  `PassMode::Direct` and lowers to an LLVM `double` return -- no string is
  consulted anywhere in that decision.
- `FnAbi { args: Box<[ArgAbi]>, ret: ArgAbi, c_variadic, fixed_count, conv,
  can_unwind }` (mod.rs:649-679); target adjusters are selected per
  `ExternAbi`/arch (`adjust_for_foreign_abi`, mod.rs:697+; Rust ABIs
  mod.rs:784+).
- `fn_abi_new_uncached` (rustc_ty_utils/src/abi.rs:543-623) is the single
  construction point: `arg_of` (abi.rs:578-596) computes
  `let layout = cx.layout_of(ty)` and wraps it in `ArgAbi::new`; the FnAbi is
  built as `ret: arg_of(sig.output(), None)` and `args: inputs.iter()...`
  (abi.rs:598-607). Then `fn_abi_adjust_for_abi` (abi.rs:619-641) and
  `fn_abi_sanity_check` (abi.rs:425-537) run. Sanity checks are strong:
  `PassMode::Direct` on a `Memory` aggregate is a hard panic
  (abi.rs:478-484), `Pair` requires ScalarPair (abi.rs:488-498).
- repr(C) matters to the ABI exactly where C compatibility requires it, and
  the flag rides on the layout: win64 consults `layout.is_repr_c()`
  (rustc_target/src/callconv/x86_win64.rs:52-64); x86_64/x86/aarch64 similar.

### 2.4 Indirect (fn-pointer) calls: return mode from the pointee signature

This is the direct answer to the study question.

- In MIR codegen, the callee layout decides everything:
  `ty::FnPtr(..) => (None, Some(callee.immediate()))` for the call target
  (rustc_codegen_ssa/src/mir/block.rs:1149).
- The signature is taken from the CALLEE VALUE'S TYPE:
  `let sig = callee.layout.ty.fn_sig(bx.tcx());` (block.rs:1209), and the ABI
  is then:
  `let fn_abi = match instance { Some(i) => bx.fn_abi_of_instance(i, extra),
  None => bx.fn_abi_of_fn_ptr(sig, extra) };` (block.rs:1217-1220).
- `fn_abi_of_fn_ptr` (rustc_ty_utils/src/abi.rs:291-297) builds
  `FnAbiDesc::for_fn_ptr` (abi.rs:242-259), whose comment is explicit:
  "Parameter attributes can never be deduced for indirect calls, as there is
  no function body available". The ABI comes entirely from the fn-pointer
  TYPE. Independent of the callsite's expression shape, independent of names,
  and independent of optimization level.
- Return-mode plumbing:
  `if fn_abi.ret.is_indirect() { llargs gets the sret pointer; llreturn is
  void }` in `FnAbi::llvm_type` (rustc_codegen_llvm/src/abi.rs:375-389);
  `make_return_dest` allocates/provides the indirect slot per
  `fn_abi.ret` (block.rs:2451-2502); `store_return` writes the returned value
  per `ret_abi` (block.rs:2505-2550).
- The call instruction's LLVM function type is built from the FnAbi:
  `let fn_ty = bx.fn_decl_backend_type(fn_abi);` (block.rs:210) and
  `fn_ptr_backend_type` is an opaque pointer (rustc_codegen_llvm/src/type_.rs:
  321-323; `ptr_to_llvm_type` = data layout's instruction address space,
  abi.rs:451-453). A debug assertion enforces the invariant:
  `debug_assert_eq!(return_slot.is_indirect(), fn_abi.ret.is_indirect(), ...)`
  (block.rs:250-254).
- Practical consequence: `let f: fn() -> f64 = transmute_or_dlsym(...); f()`
  compiles to a call whose LLVM type is `double ()*`-compatible, derived from
  `f`'s type. Floating-point returns go wherever the target ABI sends them
  (XMM0 on x86-64 SysV/Win64) and the caller reads them per the same FnAbi.
  There is no path where a caller can "declare" a different return class than
  the pointee type.
- Compatibility across declarations is itself checked with `PassMode::eq_abi`
  (mod.rs:94-118) / `ArgAbi::eq_abi` (mod.rs:~610-631), used by CFI and
  inlining decisions.

### 2.5 Monomorphisation reachability and verification (Stage 7 support)

- Only reachable instances are codegen'd: collector roots from HIR, then MIR
  use-walking expands the graph (collector.rs:1-110). This is why rustc has
  no "unused function bloats the binary with malformed bodies" class
  (REL-20261010-1854's 341 KB -> 1.9 MB observation).
- `-Z verify-llvm-ir` exists as a first-class knob
  (`pub verify_llvm_ir: bool`, rustc_codegen_ssa/src/back/write.rs:102;
  threaded at :225). The cranelift backend verifies every function
  (`docs/RUSTC-REFERENCE.md` area 8). Our analogue would be a text-IR
  structural check before clang, since our "IR" is generated text.

## 3. How XIOM does it today

All paths verified against this worktree's `crates/xiom-codegen` at
`495d6672`. Main-side fixes (m254/m256) are cited by commit and described
separately in 3.2 so the architecture picture stays accurate for both.

### 3.1 The namer architecture

XIOM has no typed layout layer on the Rust side. Types are canonicalized to
STRING names in shared registries, and emission re-derives LLVM types from
those strings and from expression shapes:

- Registries (``context.rs``): `types.types` (name -> field-name list),
  `types.type_meta` (name -> `TypeMeta { fields: Vec<(name, type-string)> }`),
  `enum_variants`, `enum_variant_field_types`, `fn_ptr_return_types`
  (context.rs:244), `local_xiom_types` / `local_vec_handle`
  (context.rs:450), plus `local_vec_elem`, `local_arr_*` maps.
- Tuple structs are named by concatenation: `Tuple__{elem0}__{elem1}...`
  (lib.rs:2032, :2079 for fn markers; decl.rs:2774; expr.rs:1166;
  lib.rs:9253). Because container names can contain `[`, `,`, `&`, `*`,
  `sanitize_container_arg` (lib.rs:2126-2136) maps those to `_` -- i.e. the
  identifier is a lossy projection of a type.
- TUPLE ELEMENTS ARE NAMED BY THREE INDEPENDENT NAMERS that must agree:
  1. the pre-registration scan `decl.rs::infer_expr_type_name` (:2927-2944)
     reached from `scan_expr_for_tuples` (:2767-2787);
  2. the literal build path `expr.rs` (:1064-1166);
  3. `lib.rs::infer_llvm_type_impl`'s tuple arm (:9195-9259).
  Each has grown its own case ladder (Bool locals, mono params, `As` cast
  targets, container ctors, mono substitutions). Agreement between them is
  maintained by comments and regressions, not by construction.
- `infer_llvm_type` (lib.rs:8777) is a recursive syntactic inference over the
  AST: literals, idents (via local maps), calls (via registered return
  types), tuples, etc. It is NOT `layout_of(Ty)`; it re-derives per call site
  and falls back to `i64` when evidence is missing.
- Enum layout: all variants share one LLVM struct; fields are the UNION of
  variant field names in declaration order plus `discriminant` (decl.rs:104-140).
  When a field NAME repeats across variants, the slot is erased to `Int`
  (i64) -- the M19 convention (decl.rs:118-127) -- and readers decode through
  `enum_variant_field_types` (stmt.rs:2618; enum_ctors.rs:47-53 for packing).
  Type identity is thus keyed by the erased field name.
- fn-pointer types survive only as strings: `fn(params) -> ret` markers
  (lib.rs:2079, types.rs:489, decl.rs:2916). `llvm_type_for` re-parses those
  strings into `{ret} ({params})*` (lib.rs:4208-4226). The `As` arm of
  `compile_expr` (expr.rs:4965-5184) emits `inttoptr iN -> {fnty}*` for
  int->fn-ptr and `ptrtoint` for the reverse.
- Fn-typed PARAMETERS get tracking: `closure_locals`, `fn_local_returns`
  (decl.rs:1727-1734) and `fn_ptr_return_types` (decl.rs:1763-1768). The last
  is RESET at every function start (decl.rs:1295). A local bound by `let f =
  ... as fn(...) -> R` gets none of these records; `infer_value_xiom_type`
  erases the cast target through `type_from_ast`, whose `Type::Fn` falls into
  the `_ => "Int"` arm (emitter.rs:443-445 + lib.rs:2012-2051).
- `parse_struct_field_types` (lib.rs:8810-8850) reads field LLVM types from
  either `type_meta` (good: registration-time record) or, as fallback, by
  splitting the tuple NAME on `_` and mapping known primitives. The fallback
  is exactly the name re-derivation class this study is about.

### 3.2 The failure modes, with the m-family as evidence

- Namers disagree -> invalid or wrong IR. m254 (main `b443e970`): all three
  namers defaulted `Expr::Index` to "Int", so `(pred[i], lab[i])` over a
  `Vec[(Float64, Int)]` built `Tuple__Int__Int`; the Float64 component was
  coerced to i64 and corrupted at runtime. The fix added the same
  `container_element_xiom_name` call to all three namers (decl.rs/expr.rs/
  lib.rs): 53 + 17 + 15 diff lines for one case.
- Erasure loses the type, then a second namer guesses. m256 (main
  `458d0afb`): a match-arm Vec payload binding whose field name repeats across
  variants is M19-erased to an i64 handle tracked only in `local_vec_handle`;
  the tuple namers saw no `local_xiom_types` entry and fell back to Int,
  emitting `Tuple__Int__Int` against the declared `Tuple__Int__Vec`
  (clang error at `ret`). The fix taught the expr.rs and lib.rs namers to
  name `local_vec_handle` bindings "Vec" and added a box->load
  materialization in the literal path.
- Inference from values instead of types. m250 (`96036551`, reverted by m253;
  still OPEN as the four smoke regressions): "call_return_xioms
  substituted-return preference" mis-bound `Option[Str]` payloads as raw
  i8*, and a receiver-offset zip inferred T from a cast's LLVM width instead
  of the cast target. The fix was reverted precisely because value-shape
  inference has non-local effects.
- fn-pointer signature erasure. REL-20261010-1904 (section 7.1) -- the
  return class is lost because the binding records neither the fn signature
  nor its LLVM type set.
- M19 (decl.rs:118-127) is the enum-level instance of the same disease:
  a shared slot is typed by ONE name-based decision (`Int`), and every reader
  must remember the per-variant truth from a side table
  (`enum_variant_field_types`). m221 was a read/write disagreement over that
  table; m256 is a consumer that did not consult it.

### 3.3 What this buys and costs

Today's design is not arbitrary: string keys make the emitter simple, make
symbols deterministic, and sit naturally on the "emit text IR and diff it"
workflow (parity/IR locks). The cost is that TYPE INFORMATION IS RE-DERIVED
FROM EXPRESSION SHAPES AT LEAST TWICE (pre-registration scan + emission) and
must be re-taught to every namer for every new expression form. rustc pays
for its typed layer with a large amount of infrastructure; XIOM can get most
of the benefit with a much smaller typed "table" layer, because we do not
need full generic layout computation -- only enough record-keeping to stop
guessing.

## 4. Gap table

+--------------------------+-------------------------------------------+--------------------------------------------+--------------------------------------+-------------------------------------------+
| Concern                  | rustc                                     | XIOM today                                 | Failure class                        | Direction                                  |
+--------------------------+-------------------------------------------+--------------------------------------------+--------------------------------------+-------------------------------------------+
| Type identity            | interned TyKind (ty/mod.rs:517);          | string keys + name joining                 | collisions, lossy sanitize           | intern Id; render only at emission         |
|                          | Instance(def,args) (instance.rs:33)       | (lib.rs:2032, 2126-2136)                   | (sanitize_container_arg)             |                                            |
+--------------------------+-------------------------------------------+--------------------------------------------+--------------------------------------+-------------------------------------------+
| Tuple element type       | layout field Ty per variant               | three namers must agree                    | m254 (Index default Int), m250/m256  | one typed tuple constructor used by all    |
|                          | (ty/layout.rs:848)                        | (decl.rs:2927, expr.rs:1070, lib.rs:9197)  |                                      | three call sites                           |
+--------------------------+-------------------------------------------+--------------------------------------------+--------------------------------------+-------------------------------------------+
| Enum field type          | per-variant VariantLayout; niche tags     | union of names + M19 Int erasure           | m221, m256, M19 family               | per-variant field records; keep Int slot   |
|                          | (rustc_abi/src/lib.rs:2032-2111)          | (decl.rs:104-142)                          |                                      | only as ABI packing with typed readers     |
+--------------------------+-------------------------------------------+--------------------------------------------+--------------------------------------+-------------------------------------------+
| Aggregate layout source  | LayoutData (size/align/offsets/repr_c)    | type_meta strings + parse_struct_field_    | invalid stores/loads; clang rejects  | store offsets+field LLVM types at          |
|                          | (rustc_abi/src/lib.rs:2223)               | types fallback split (lib.rs:8810)         |                                      | registration; delete the splitter          |
+--------------------------+-------------------------------------------+--------------------------------------------+--------------------------------------+-------------------------------------------+
| Fn-pointer return mode   | FnAbi ret from fn_abi_of_fn_ptr           | ret_ty falls back to i64; hardcoded i64    | REL-20261010-1904 (fp returns)       | record fn_sig at binding; derive ret from  |
|                          | (rustc_ty_utils/src/abi.rs:291-297)       | in 3 call paths (call.rs:6089-6112,        |                                      | it in every call emission path             |
|                          |                                           | :938-942, lib.rs:6146-6150)                |                                      |                                            |
+--------------------------+-------------------------------------------+--------------------------------------------+--------------------------------------+-------------------------------------------+
| ABI sanity               | layout_sanity_check (invariant.rs:10);    | none (clang is the only check)             | invalid IR reaches clang; m256       | always-on IR structural check + CI        |
|                          | fn_abi_sanity_check (abi.rs:425)          |                                            |                                      | clang -fsyntax-only for fixtures           |
+--------------------------+-------------------------------------------+--------------------------------------------+--------------------------------------+-------------------------------------------+
| Opt-level independence   | layout/ABI computed once, opt-independent | emission can differ by inference path      | Stage 7 differential gate            | typed table + IRL locks enforce sameness   |
+--------------------------+-------------------------------------------+--------------------------------------------+--------------------------------------+-------------------------------------------+

## 5. Optimization & performance opportunities

1. String round-trips (Stage 6). Tuple names are built by `join("__")` and
   re-parsed by `parse_struct_field_types`; container names are sanitized and
   re-split for element types; `fn_type_return_xiom` (vec_abi.rs:547-555)
   re-parses the return out of an "fn(...) -> R" spelling that the codegen
   wrote itself. A small interned `TypeRef` (Rc/Arc or arena id) with a
   `render()` used ONLY at the emission boundary removes the allocate/parse
   cycles. Because `render()` must reproduce current spellings, the existing
   IR/parity locks stay valid.
2. `infer_llvm_type` is called per use site and recurses through calls to
   registered return types (lib.rs:9154 loops over all `types.functions`
   entries in the suffix fallback). A per-function memo keyed by expression
   span, or better by the typed table, removes repeated scans.
3. The pre-registration scan (`register_expr_tuple_types`,
   decl.rs:2683-2716) walks the whole body before emission; the tuple literal
   path then re-derives the same names during emission. Consolidating to one
   typed pass halves AST traversal and, more importantly, removes the
   possibility that the two traversals see different environments (the m250
   leak class).
4. Layout data (size/align) is recomputed ad hoc (`vec_elem_storage_size`,
   `array_type_bytes`, `alloca_align`). A per-type layout record computed at
   registration time gives one source; cache it across monomorphisations.
5. Determinism (R25/R39 policy) gets EASIER with a typed layer: no
   suffix-search over HashMap keys (lib.rs:4145-4155, call.rs:5766) means no
   need for `pick_deterministic` tie-breaking; identity is the type itself.
6. Do not over-build: XIOM does not need rustc's niche computation or layout
   randomization. The minimum viable typed layer is (a) container element,
   (b) tuple elements, (c) per-variant enum fields, (d) fn signatures,
   (e) aggregate field offsets/LLVM types.

## 6. Adoption proposals (byte-identical IR preserved)

Migration principle: the typed layer is INTERNAL; the emitted text is a pure
function of it, and a compatibility renderer must reproduce today's exact
spellings. Land the layer with a debug assertion
`render(type_id) == existing_name` at every registry insert, run the IR locks,
and only then delete the old derivation paths.

Phase 0 -- one naming API, no IR change (days, low risk)
  - Add `TypeRef` (interned) for the four families: scalar, container(elem),
    tuple(Vec<TypeRef>), fn(params, ret), enum(name, variant fields).
  - Replace the three tuple namers with
    `tuple_type_ref(elems) -> TypeRef` + `render`; keep the exact
    `Tuple__A__B` spelling (including the R60 bare-base policy for nested
    containers and the sanitize rules) inside `render` only.
  - Debug-assert agreement between the scan path and the emission path for
    every tuple registered, with a message naming the expression span.
  Acceptance: full IR locks green; no diff in emitted test fixtures.

Phase 1 -- typed locals and fn signatures (fixes REL-20261010-1904 shape)
  - `LocalTypeTable`: `name -> TypeRef`, populated at Let/Var (stmt.rs:241),
    match bindings (stmt.rs:2618, 2993), params (decl.rs:1700-1768), loop
    vars (stmt.rs:187), inherited bindings (stmt.rs:601).
  - Fn-ptr locals/params record `fn(params, ret)` from the `As` TARGET
    (`Type::Fn`, expr.rs:4965 context) and from `Type::Fn` params; write it
    into `fn_ptr_return_types`-equivalent (or a new typed map) at binding
    time.
  - Call emission consults the record: `ret_llvm` and parameter LLVM types
    come from the fn signature, mirroring `fn_abi_of_fn_ptr`. Apply in all
    three places: `callee_is_fn_ptr` path (call.rs:6084-6114), the
    complex-callee path (call.rs:903-943), and `compile_index_fn_ptr_call`
    (lib.rs:6119-6152).
  - `type_from_ast(Type::Fn)` must stop returning "Int": route through
    `type_from_ast_with_args` (lib.rs:2079) so bindings keep the marker.
  Acceptance: new IR lock asserting `call double` and the real `{ret}
  ({params})*` fn type for the glfw-shaped fixture; e2e with a real fp
  function.

Phase 2 -- layout authority for aggregates
  - At type registration, compute and store per-type: `size`, `align`,
    `Vec<(field_index, field_llvm_ty, offset)>` (a mini LayoutData). Emit the
    struct definition and every GEP from this record.
  - Replace `parse_struct_field_types` fallback with the record; keep the
    function but make it consult only the record (then delete the `_` split).
  Acceptance: IR locks for struct GEPs unchanged; remove the split fallback
  with a grep proving no callers rely on unregistered tuples.

Phase 3 -- enum payloads typed per variant
  - Store `variant_field_types: Vec<Vec<TypeRef>>` (already logically present
    in `enum_variant_field_types`, stmt.rs:2618) as structured records, and
    make constructor packing (enum_ctors.rs:54-82) and match decoding use the
    same record object (not a parallel string table).
  - Keep the shared i64 slot and M19 erasure as the ABI representation, but
    record the ERASURE DECISION in the record (slot type + decode type), so a
    consumer can never silently disagree (m221/m256 classes).
  Acceptance: m221/m256 fixtures green plus new IR assertions on the slot
  store/decode widths.

Phase 4 -- verification and differential gates (Stage 7)
  - Before invoking clang, run a text-IR structural check: every `call`
    result/arg types match the callee declaration; every `store` type matches
    the alloca/GEP pointee; report the first mismatch with source span (the
    XIOM analogue of fn_abi_sanity_check).
  - Add the fp-fn-ptr differential fixture to the -O0/-O2/-O3 and clang/gcc
    matrix; assert identical program output.
  - CI: keep the existing IR locks as the byte-identical guard; the typed
    layer must be provably render-compatible (a script diffing names from
    both layers over the test corpus).

## 7. Open-item relevance

### 7.1 REL-20261010-1904: Float64 returns through cast fn-pointers

Bus item: `REL-20261010-1904-bindings-2` (bindings -> compiler, bug, acked).
Repro: `let f = lib.p_sym as fn(...) -> Float64; f()` -- glfwGetTime never
advances (0 after glfwSetTime(99.0)); glfwGetWindowOpacity reads 262416.0
instead of 1.0. Integer/pointer returns and fp OUT-params are correct;
RETURNS only. Same DLL via ctypes works.

Where the fp return is lost (verified trace through this worktree's code):

1. `let f = <sym> as fn() -> Float64`:
   - `Expr::As` (expr.rs:4965) computes
     `target_llvm_ty = llvm_type_for_fallback(type_from_ast(Type::Fn))`
     = `double ()*` (fn-string parsing, lib.rs:4208-4226) and emits
     `inttoptr i64 %sym to double ()*` (expr.rs:5166-5174), returning the
     value typed `double ()*`. So far correct.
   - The binding (stmt.rs:241) has no annotation: `declared_llvm_ty = None`,
     slot = `double ()*`. But `infer_value_xiom_type` (emitter.rs:443-445)
     records `local_xiom_types["f"] = type_from_ast(Type::Fn)` and
     `type_from_ast` has NO `Type::Fn` arm -- it hits `_ => "Int"`
     (lib.rs:2051). The return type is ERASED AT BINDING TIME. Nothing writes
     `fn_ptr_return_types` for value bindings (only `Type::Fn` params do:
     decl.rs:1763-1768; the map is also reset per fn: decl.rs:1295).
2. `f()`:
   - `ret_ty` (call.rs:5752-5778) looks in `types.functions` for "f"; no
     entry, no qualified/suffix struct-return match -> fallback `i64`.
   - `callee_is_fn_ptr` (call.rs:5890-5896) fires because the local exists,
     is not a closure, no function is registered, AND `ret_ty == "i64"`. Note
     the guard itself encodes the bug: a correct `double` ret_ty would not
     even take this path.
   - `actual_ret_ty` (call.rs:6089-6093) is `fn_ptr_return_types.get("f")` ->
     None -> `i64`.
   - Emitted IR: `bitcast double ()* %loaded to i64 ()*; call i64
     %fnptr()`. The real callee returns the double in XMM0; the caller reads
     RAX. Garbage/0 depending on what the callee left in RAX -- exactly the
     reported "never advances / 262416.0".
   - The same loss exists in the other fn-pointer call paths:
     complex callee (call.rs:938-942) hardcodes `i64 (...)`, and
     `compile_index_fn_ptr_call` (lib.rs:6146-6150) hardcodes `i64`.
   - The annotated spelling `let f: fn() -> Float64 = ...` is ALSO broken:
     the slot is erased to i64 (stmt.rs:527-533) but the call still uses the
     i64 fallback for the return type.
3. Why integer/pointer returns work: on x86-64 SysV/Win64 both RAX and the
   integer register file coincide with the `i64 (...)*` guess, so the wrong
   LLVM signature is accidentally ABI-compatible. Float-class returns are the
   first class where the guess is observably wrong; small struct returns
   (register pairs / sret) are the next.

Fix sketch (Phase 1 above, minimal first patch):
- Make `type_from_ast(Type::Fn)` (and `infer_value_xiom_type`'s As arm)
  preserve the `fn(...) -> R` marker, and write the fn signature record
  (params + ret as XIOM types) at every binding that carries a `Type::Fn` or
  a cast-to-Fn target.
- In all three call paths, derive `{ret} ({params})*` from the record via
  `llvm_type_for`; only fall back to i64 when no record exists (log a
  counter; it must reach 0 on the corpus).
- Regression: IR lock `call double` + e2e against a real dynamic library
  returning a double (the bindings lane's glfw fixture is the acceptance
  test); add to the -O0/-O2/-O3 matrix.

### 7.2 m250 / m254 / m256 / M19 in one paragraph each

- m254 (main `b443e970`, "tuple-literal indexed element keeps its XIOM
  type"): all three tuple namers defaulted `Expr::Index` to Int; a
  `Vec[(Float64, Int)]` element read became the Int component and the
  Float64 payload was corrupted. Fix: one new resolver
  (`container_element_xiom_name`) called from all three. This is the
  "N namers, N patches per bug" cost made explicit.
- m256 (main `458d0afb`, "match-arm Vec binding loses its container type in
  tuple returns"): M19 erasure (decl.rs:118-127) + a binding tracked only in
  `local_vec_handle` meant the namers had no type and produced
  `Tuple__Int__Int` against `Tuple__Int__Vec` (clang ret error). Fix: namers
  consult `local_vec_handle`; literal path materializes the handle through
  the box. A typed per-variant record would remove the "tracked only in a
  side map" failure mode.
- m250 (`96036551`, reverted by m253; still OPEN as four stdlib smoke
  regressions at the time of this study): value-shape inference of substituted
  returns and receiver element types mis-typed Option[Str] payloads and
  narrow-int receivers. Recovered only by revert; the implementation notes
  live in the commit. This is the strongest argument that inference from
  expression shapes (rather than binding-time records) cannot be made
  incrementally correct.
- M19 (decl.rs:118-127): shared enum slots are erased to i64 whenever a field
  name repeats across variants, with per-variant truth in
  `enum_variant_field_types`. It works only as long as every writer/reader
  remembers the side table (m221 was a packing-width disagreement; m256 a
  missing lookup). rustc's `Variants::Multiple`/`VariantLayout` model
  (rustc_abi/src/lib.rs:2032-2111) is the reference shape: the slot layout is
  derived per variant, and field identity is (variant, index), never a name.

### 7.3 Stage 7 tie-in

The differential -O0/-O2/-O3 contract (COMPILER_READINESS_PLAN.md Stage 7)
is exactly the invariant rustc gets for free: layout/ABI are opt-independent
because they are computed from types once and shared. REL-1904 is an
ABI-class divergence that optimization level does not even trigger -- it is
wrong at every -O. Closing it by storing types instead of guessing them is
therefore a direct Stage 7 investment, and the always-on IR structural check
(Phase 4) converts the whole m-family from "clang rejects / runtime garbage"
into "compiler error at the first mismatched store or call".

## 8. Sources

rustc (clone E:\repos\rust @ 7211ea594cbf137ef6118b1752b959dea7cb9088):
- compiler/rustc_middle/src/ty/mod.rs:517 (interned Ty).
- compiler/rustc_middle/src/ty/instance.rs:23-96 (Instance, InstanceKind).
- compiler/rustc_middle/src/ty/layout.rs:804, 848 (variant/field layout bridge).
- compiler/rustc_ty_utils/src/layout.rs:40, 207 (layout_of).
- compiler/rustc_ty_utils/src/layout/invariant.rs:10 (layout_sanity_check).
- compiler/rustc_ty_utils/src/abi.rs:242-259 (for_fn_ptr), 291-297
  (fn_abi_of_fn_ptr), 425-537 (fn_abi_sanity_check), 543-623
  (fn_abi_new_uncached; ArgAbi construction and adjust).
- compiler/rustc_abi/src/lib.rs:2032-2111 (Variants, TagEncoding), 2113-2137
  (Niche), 2223-2320 (LayoutData, largest_niche, repr_c).
- compiler/rustc_abi/src/layout.rs:395-527 (layout_of_union, repr_c
  propagation), 529-581 (layout_of_struct), 583+ (layout_of_enum).
- compiler/rustc_abi/src/layout/ty.rs:76-150 (TyAndLayout, for_variant, field).
- compiler/rustc_target/src/callconv/mod.rs:59-92 (PassMode), 394-424
  (ArgAbi/ArgAbi::new), 649-679 (FnAbi), 697-782 (adjust_for_foreign_abi),
  784+ (adjust_for_rust_abi).
- compiler/rustc_target/src/callconv/x86_win64.rs:8-74 (win64 fixup; repr_c
  ZST/return handling).
- compiler/rustc_codegen_ssa/src/mir/block.rs:170-254 (do_call; fn_decl type;
  indirect-slot debug assert), 1149, 1209-1259 (fn-ptr signature and FnAbi
  selection; indirect return slot), 2451-2502 (make_return_dest), 2505-2550
  (store_return).
- compiler/rustc_codegen_ssa/src/traits/builder.rs:39-49 (ReturnSlot).
- compiler/rustc_codegen_ssa/src/back/write.rs:102, 225 (verify_llvm_ir).
- compiler/rustc_codegen_llvm/src/abi.rs:364-449 (FnAbi::llvm_type; return
  mode lowering), 451-453 (opaque fn-ptr type).
- compiler/rustc_codegen_llvm/src/type_.rs:318-323 (fn_decl/fn_ptr types).
- compiler/rustc_monomorphize/src/collector.rs:1-110 (reachability graph).

XIOM (this worktree @ 495d6672 unless noted):
- docs/COMPILER_READINESS_PLAN.md:201-210 (Stage 7).
- docs/COMPILER_ARCHITECTURE.md:335-395 (codegen context).
- docs/RUSTC-REFERENCE.md (main worktree; areas 4-9 and active-item map).
- crates/xiom-codegen/src/context.rs:244, 261, 450 (registries).
- crates/xiom-codegen/src/decl.rs:98-148 (enum registration, M19),
  1255-1352 (per-fn setup, ret_llvm), 1700-1768 (param tracking),
  2683-2816 (tuple pre-registration scan), 2927-2944 (infer_expr_type_name).
- crates/xiom-codegen/src/expr.rs:1064-1224 (tuple literal path), 4965-5184
  (As casts incl. fn-ptr), 5873 (scrutinee type inference).
- crates/xiom-codegen/src/vec_abi.rs:210, 356 (tuple string-prefix checks),
  547-563 (fn_type_return_xiom / is_fn_marker).
- crates/xiom-codegen/src/lib.rs:2012-2053 (type_from_ast; Type::Fn -> Int),
  2079 (fn marker), 2126-2136 (sanitize_container_arg), 4036-4242
  (llvm_type_for; fn-string parsing at 4208-4226), 6119-6152
  (compile_index_fn_ptr_call), 8777-8850 (infer_llvm_type,
  parse_struct_field_types), 9154-9259 (fn-ptr return lookup; tuple arm).
- crates/xiom-codegen/src/call.rs:5752-5778 (ret_ty fallback), 5883-5896
  (callee_is_fn_ptr), 5906-5951 (global fn field), 5965-6038 (instance fn
  field), 6084-6114 (fn-ptr call emission; actual_ret_ty).
- crates/xiom-codegen/src/stmt.rs:241-689 (Let binding; fn-typed slot erasure
  at 527-533), 2618-2710 and 2993-3000 (match bindings, local_vec_handle).
- crates/xiom-codegen/src/enum_ctors.rs:39-86 (per-variant packing), 47-53
  (enum_variant_field_types lookup).
- crates/xiom-codegen/src/emitter.rs:443-445 (infer_value_xiom_type).
- OLDER-BRANCH NOTE: main's m254 = commit b443e970 (decl.rs +53,
  expr.rs +17, lib.rs +15), m256 = commit 458d0afb (expr.rs +41, lib.rs +8);
  both verified via `git show` from the main worktree. m250 = 96036551,
  m251 = 0d4ca07d, revert m253 per docs/COMPILER_BUGS.md:14801-14832
  (main worktree).

Bus:
- E:\xiom-lang\xiom-relays\items\REL-20261010-1904-bindings-2.md (the
  fp-return bug; refs xiom-packages/docs/repro/bindings-pilot/glfw-fp-return).
- E:\xiom-lang\xiom-relays\items\REL-20261010-1904-bindings.md (batch context;
  B-12 finding referenced by the same lane).
