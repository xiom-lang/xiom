<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# ARC A -- Real Pointer/Reference Types (production design)

**Goal:** `*T`/`*mut T`/`*const T` and `&mut T` become real LLVM pointer types end-to-end, so
deref (`*p`), address-of (`ptr.from_ref`/`from_mut`, `&x`), and mutation-through-pointer work.
Unblocks stdlib exec: `sync` (Arc `count: *Int` field deref), `serialize` (`pos: &mut Int`),
`mem` (`swap(&mut a,&mut b)`), `path`, `crypto`, `regex`.

## Investigation findings (verified 2026-07-09)
- `extern_type_to_llvm` (lib.rs ~527) ALREADY maps `Ptr/Ref/MutRef(inner)` -> `<inner>*` correctly.
  It's used for `extern "C"` decls -- which is why FFI pointer args already work.
- `type_from_ast` (lib.rs ~567) DROPS pointer-ness: `Ref/MutRef -> inner name`, `Ptr -> "Int"` (the
  `_ =>` arm). Field types (`type_meta.fields`, registered ~1553 via `type_from_ast`) and param types
  (via `llvm_type_for_fallback(type_from_ast(..))`) therefore collapse `*Int`->i64, `*Float32`->i64.
- Result: a `*Int` struct field or `&mut Int` param is stored as `i64`, then deref emits
  `load i64, i64* %x` where `%x` is an `i64` -> clang "i64 but expected ptr" (sync) or a crash.
- BLAST RADIUS: 186 `&`/`*` params across examples. **KEY:** almost all are `&StructType`/`&Vec[T]`
  which currently "work" by-value because the struct/Vec carries its heap pointer by value. Do NOT
  change `&Struct`/`&Vec` behavior. The genuinely-broken cases are **pointer-to-SCALAR**:
  `*Int`/`*Float32`/`*UInt8` fields & locals, and `&mut Scalar` params.

## DESIGN (minimize blast radius; keep the 410 gate green)

### Canonical pointer encoding through the type-name pipeline
Introduce a reversible pointer marker in the `type_from_ast` string so field/param/local type
resolution preserves pointer-ness without touching the 186 `&Struct` sites.
- `type_from_ast(Type::Ptr(inner))`   -> `"*" + type_from_ast(inner)`  (e.g. `*Int`, `*Float32`)
- `type_from_ast(Type::MutRef(inner))` -> keep CURRENT behavior (inner name) for `&mut Struct`/`&mut Vec`
  BUT for a scalar inner (`Int/Char/Bool/Float*/UInt*`) -> `"*" + inner` (by-ref scalar).
- `type_from_ast(Type::Ref(inner))`    -> **unchanged** (inner name) -- `&T` stays by-value-carrying.
- `llvm_type_for(name)`: if `name` starts with `*`, return `llvm_type_for(name[1..]) + "*"`
  (e.g. `*Int`->`i64*`, `*Float32`->`float*`, `*UInt8`->`i8*`). Everything else unchanged.
- This makes `field_llvm_type` and param typing yield real pointers for `*T`/`&mut Scalar` with ZERO
  change to struct/Vec ref handling.

### Deref read/write (`*p` and `*p = v`)
- `Expr::Unary(Deref, p)` / `Expr::Deref`: compile `p` (type `T*`), emit `load T, T* p` -> `(reg, T)`.
- `Stmt::Assign` with `Expr::Deref(p)` place (or `(*raw).field = v`): compile `p`->`T*`, `store T v, T* p`.
  Handle `(*raw).field = v`: GEP into the pointed-to struct then store.

### Address-of (`&x`, `ptr.from_ref(x)`, `ptr.from_mut(x)`)
- `ptr.from_ref`/`from_mut(x)`: if `x` is an lvalue (local/param/field), return its ADDRESS (alloca/GEP)
  typed `T*`. If `x` is already a `T*` (a by-ref param), return it as-is (identity).
- `&x`/`&mut x` (`Expr::Ref/MutRef`) at a CALL SITE where the callee param is a scalar-ref (`*T` per
  encoding): emit the address of `x`. Otherwise (callee wants `&Struct` by value): keep current
  compile-the-value behavior. Use the callee's registered param type to decide (call-arg coercion
  already fetches `callee_pts`).

### Param binding for `*T`/`&mut Scalar` params (compile_fn + generic-mono)
- The incoming `%paramN` is already a `T*`. Bind the param name to a local slot of type `T*` holding
  the incoming pointer (i.e. `alloca T*; store T* %paramN; bind name->(slot, "T*")`). Then:
  - a plain read of the param loads the pointer (`load T*`) -- correct for passing it on;
  - `ptr.from_ref(param)`/`*param` deref through it;
  - to READ the pointee value where the param is used as a value, deref.
  (Reuse existing local-slot machinery; the local's declared type is `T*`.)

### Registration
- `register_functions` param-type list + generic-mono specialized param types must use the same
  pointer-encoded -> llvm mapping so call-arg coercion targets match (pointer params get `T*`).

## IMPLEMENTATION ORDER (gate-check EACH step; revert if red)
1. `type_from_ast` Ptr/scalar-MutRef encoding + `llvm_type_for` `*`-decode. Build; run FULL gate
   (`cargo test -p xiom-codegen`, `-p xiom-check`, `-p xiom-parser`). This alone changes field/param
   LLVM types for `*T`/`&mut Scalar` -- verify no regression (should be inert for existing passing code
   since they don't use scalar pointers in the broken way).
2. Deref read/write through real pointers.
3. from_ref/from_mut address-of + identity-on-pointer.
4. `&x` address-of at scalar-ref call sites (decided via callee param type).
5. Param binding for pointer params (compile_fn + generic-mono + registration).
6. Verify sync, serialize, mem, then path/crypto/regex. Full gate green.

## TESTS (add as you go)
- `examples/e2e/ptr_deref.xi`: `let x=5; let p = ptr.from_ref(x); *p == 5` -> 0.
- `examples/e2e/ref_mut_param.xi`: `fn inc(p: &mut Int){ *p = *p + 1; } var a=1; inc(&mut a); a==2` -> 0.
- `examples/e2e/ptr_field.xi`: struct with `*Int` field, deref it.
- feature_regression: `*T` param/field parse+emit.
- Then stdlib exec sync/serialize/mem flip to strict green.

## RULE
The 410 gate (codegen+check+parser) MUST stay green after every step. If a step can't stay green,
revert THAT step, narrow, and document. A green 24/36 beats a red 27/36.
