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
