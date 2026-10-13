# Failed Attempts Log

Per the core protocol's Circuit Breaker: after 3 failed attempts on one issue,
stop, log here, and escalate.

## 2026-09-19 -- L5-40 `Map[Int, Vec[Str]]` container-payload ABI (playground C17 residue)

**RESOLVED 2026-09-21 (R56).** The coordinated fix landed: nested type-arg
rendering in BOTH explicit-arg fallbacks (`type_arg_to_name` + mono
substitution), container-typed bare-local generic inference
(`infer_generic_ident_type`, with fixed-array brackets explicitly excluded),
container acceptance in `record_field_vec_elem`, container lowering for
substituted names in the mono signature builder, and concrete-Option-aware
match payload binding. Decision: container elements are INLINE
(`Vec[Str]` slots hold the 32-byte `%struct.Vec` header -- matching
`Vec[Vec[T]]` and `vec_elem_storage_size`); concrete Option/Result layouts
keep inline payloads and consumers resolve fields through the concrete
registration. L5-40 prints 2; lock `e2e_m111_map_vec_container_payload`; full
e2e 2359/2359. Details in the COMPILER_BUGS R56 entry.

Symptom: `group_by_age` prints 0 (expected 2); the reduced probe
`Map[Int, Vec[Str]]` prints `1` for `m.len()` and a garbage length for the
retrieved Vec (`m.get(&25)` match payload).

Attempts:
1. Registered the concrete nested `Option__Vec_Str_` at the call site and in
   the mono definition (kept -- this fixed the original clang "Cannot
   allocate unsized type" failure; L5-40 now builds).
2. Fixed the index-assign handler's struct-element store and the `unwrap`
   box/unbox pairs (kept -- fixed L6-28/L8-15/L8-18 and the concrete
   `Vec[Task].get().unwrap()` probe).
3. Fixed the explicit receiver type-arg rendering for NESTED containers
   (`Map[Int, Vec[Str]].new()` inferred V=Int because `Expr::Index` args
   rendered as "Int"; now uses `type_arg_to_name`) and added mono-substituted
   element resolution to the Vec index path. Result: the constructor now
   monomorphises as `Map.new_Int_Vec_Str_` (was `new_Int_Int`), but the probe
   still mis-reads the payload, and an additional attempt to record
   implicit-self Vec FIELDS with container element types
   (`record_field_vec_elem` accepting `Vec[...]`) made both the probe and
   L5-40 WORSE (garbage / AV) and was reverted.

Current precise state (evidence: `tmp/p_map_vec_final.exe.ll`, kept local):
- `Map.new_Int_Vec_Str_` constructs `values: Vec[V]` (element convention
  must be one of: inline %struct.Vec headers with esz 24, or 8-byte box
  handles -- the probe shows esz 8 for the generic field).
- `Map.get_Int_Vec_Str_` reads `values[i]` through the scalar `emit_elem_load`
  switch (8-byte load) and then inttoptr+loads the value as `%struct.Vec`.
- `Map.insert_Int_Vec` stores the value with a runtime-`esz` memcpy of the
  header alloca (inline-header convention).
The two conventions disagree; committing to one across
`record_field_vec_elem`, the index path, `emit_elem_payload_load`, push and
the Option/Result ctor is a cross-cutting change that needs its own design
pass (touches Map/Vec/Option container ABI), not a surgical fix.

Escalation: needs an architecture decision on the single element
representation for container-typed Vec elements (inline vs handle), then a
coordinated change with the full e2e as the gate.

---

## 2026-10-13 -- m288 `REL-20261010-1548-bindings-4` module-leaf alias collision (checker import resolution)

Four fix attempts on the checker's import registration, all reverted (the
tree is clean at m287; no m288 batch landed):

1. Attached the direct-load branch's module under its PARENT surface
   (`modules["xiom"]["ffi"]`) so `module_exports_for_alias` would not fall
   back to the leaf map. No effect on the repro.
2. Ran the G-32 bare-item injection loop in the direct-load branch.
   `imported_has_c_strlen` stayed false: `c_strlen` is declared in the
   SUBMODULE `xiom.ffi.c` (file `stdlib/xiom/ffi/c.xi`,
   `module xiom.ffi.c` parses as nested x{ffi{c}}), so the `xiom.ffi`
   surface only carries a "c" SubModule entry; a top-level iteration never
   sees the fn.
3. Registered `cached.program.items` signatures in the direct-load branch.
   `functions` still had no `xiom.ffi.c.c_strlen` -- the registration did
   not persist from that branch.
4. Dropped the parent attach again and re-ran: unchanged. The
   `use xiom.ffi.c` processing takes a NON-registering route in the broken
   run regardless.

Current precise state (traces env-gated `XIOM_TRACE_IMPORT`, scratch in
`%TEMP%\kilo\m288`; builds kept local, all reverted):
- Repro: `module probe.ffi` + `use xiom.ffi;` + `c_strlen(1)` -> T001
  undefined variable (8:10). Control with module `probe.ffx`: compiles 0.
  `ffi.c.c_strlen` (the correct qualified spelling) compiles 0 with
  attempt 3 applied.
- Divergence point: at the first use, `modules["xiom"]` has **7 keys, ffi
  absent** in the failing run vs **12 keys, ffi present** in the control;
  the failing run takes the direct-load branch, and the later
  `use xiom.ffi.c` never registers c.xi's fns. In the control,
  `register_fn_signature` fires with `module_path='xiom.ffi.c'` =>
  `key='xiom.ffi.c.c_strlen'` (catalog=false) exactly once; in the failing
  run it never fires.
- `value_member_resolves("ffi")` false; the error surfaces from the call
  fallback, so resolution never reaches the loaded module's signatures.

Escalation: needs a decision on WHERE submodule signatures get registered
for a directory module (the normal walk's found-path does not register;
only direct loads do), or a pre-registration of the full nested module
tree when a collision routes an import through the direct-load branch.
Not a surgical fix -- start from the `use xiom.ffi.c` route divergence.

---

## 2026-10-13 -- m291 `REL-20261010-1548-stdlib-6` leg b by-value struct UFCS (codegen call resolution)

One fix attempt on the codegen call resolution, reverted (tree clean at
m290; leg a is already fixed in m290/bc716bca):

1. At the instance-receiver resolution in call.rs, added a reroute: when
   the receiver-derived key ("JsonValue.json_get_path") has no symbol and
   exactly one same-leaf key HAS one, bind that key. No effect -- tracing
   showed `types.functions.contains_key("JsonValue.json_get_path")` is
   FALSE and there is NO key ending ".json_get_path" at all in codegen's
   function map for this program (`sameleaf=[]`).

Findings that matter for the next attempt (traces env-gated
XIOM_TRACE_FMC, scratch `%TEMP%\kilo\m290`):
- The qualified call `sjson.json_get_path(v, &path)` works through the
  MODULE-receiver path (a free-fn/module surface + the m142 canonical
  direct-emit for module-qualified stdlib calls, call.rs ~1005), NOT
  through `types.functions` -- the emitted call resolves without any
  "*.json_get_path" key existing.
- `catalog_resolved_calls` (the checker->codegen handoff) is recorded
  ONLY when `checking_catalog` (catalog bodies); a USER-program
  method-style call has no recorded target, and `resolve_catalog_call`
  also validates the receiver TEXT against the target module, which an
  instance receiver ("v") can never satisfy.
- `program_bare_targets` covers BARE calls only.
- The checker accepts the call (it resolves the free fn in its own
  registry), so the gap is purely the checker->codegen binding for
  method-style calls on by-value struct receivers.

Next: either extend the checker's resolved-call recording to user-program
method-style calls resolved via UFCS (hand codegen a target it can bind
without receiver-text validation), or teach codegen the same param0-type
UFCS candidate search the checker uses. Not surgical yet -- needs the
handoff decision.
