// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashMap;
use xiom_ast::*;

impl super::IrEmitter {
    /// m283 (REL-20261010-1549-packages-8): name of the current function's
    /// contract re-entrancy depth counter.
    pub(crate) fn contract_counter_name(&self) -> Option<String> {
        let fn_name = self.fctx.current_fn.clone()?;
        let mut sanitized: String = fn_name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
            .collect();
        if sanitized.is_empty() || sanitized.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            sanitized = format!("f_{sanitized}");
        }
        Some(format!("__xiom_ctype_{sanitized}"))
    }

    /// True when the current function has any requires/ensures clause (the
    /// guard machinery is only emitted for those).
    pub(crate) fn fn_has_contract_clauses(&self) -> bool {
        !self.fctx.current_contract_clauses.is_empty()
    }

    /// Emit the function-entry increment for the contract re-entrancy guard.
    ///
    /// m328: the entry load is ALSO the re-entrancy test every clause site
    /// branches on. `d` is the depth BEFORE this invocation increments (0 for
    /// the outermost invocation, >= 1 for a re-entrant one). This
    /// invocation's clause sites always observe `d + 1` -- only this function
    /// writes its own thread-local counter and nested invocations rebalance --
    /// so `d > 0` is exactly the m291 site test `counter > 1`. Computing it
    /// ONCE here removes the per-clause-site load+icmp pair from every check
    /// (half of the m328 fmt_peek IR budget regression).
    pub(crate) fn emit_contract_guard_enter(&mut self) {
        let Some(counter) = self.contract_counter_name() else { return };
        self.declare_contract_counter(&counter);
        let d = self.fresh_tmp();
        self.emitln(&format!("  {d} = load i64, i64* @{counter}"));
        let d1 = self.fresh_tmp();
        self.emitln(&format!("  {d1} = add i64 {d}, 1"));
        self.emitln(&format!("  store i64 {d1}, i64* @{counter}"));
        let nested = self.fresh_tmp();
        self.emitln(&format!("  {nested} = icmp sgt i64 {d}, 0"));
        self.fctx.contract_nested_tmp = Some(nested);
    }

    /// Declare the guard counter at first use. Emitted into THIS function's
    /// output buffer, before its first instruction (the parallel-codegen path
    /// compiles functions into per-function buffers, so a module-level flush
    /// would land in the wrong buffer -- `use of undefined value` for
    /// late/parallel functions). LLVM hoists a leading global declaration.
    fn declare_contract_counter(&mut self, counter: &str) {
        if self.local.contract_eval_counters.insert(counter.to_string()) {
            self.emitln(&format!("@{counter} = internal thread_local global i64 0"));
        }
    }

    /// Emit the return-path decrement for the contract re-entrancy guard.
    /// Gated EXACTLY like `emit_contract_guard_enter` (function-decl extent
    /// clauses), so enter/leave stay balanced per function.
    pub(crate) fn emit_contract_guard_leave(&mut self) {
        if !self.config.check_contracts || !self.fn_has_contract_clauses() {
            return;
        }
        let Some(counter) = self.contract_counter_name() else { return };
        let d = self.fresh_tmp();
        self.emitln(&format!("  {d} = load i64, i64* @{counter}"));
        let d1 = self.fresh_tmp();
        self.emitln(&format!("  {d1} = sub i64 {d}, 1"));
        self.emitln(&format!("  store i64 {d1}, i64* @{counter}"));
    }

    /// Emit a runtime check for a single boolean contract expression.
    /// If the expression evaluates to false (i64 0), emit a panic and trap.
    ///
    /// m283 (REL-20261010-1549-packages-8): re-entrancy guard. When the current
    /// function is already invoked higher on the stack (depth > 1), this clause
    /// is being evaluated from INSIDE one of the function's own contract checks
    /// (a postcondition call-cycle: `f`'s ensures calls `g` whose contract
    /// calls `f`...). Evaluating it again recursed to the depth-500 trap
    /// (0xC000001D, no diagnostic). A re-entrant clause check is SKIPPED -- the
    /// outermost invocation (depth == 1) still runs -- while the bounded call
    /// recursion inside the clause remains the ordinary recursion counter's job.
    /// m291 fix: the m283 skip test was `depth > 0`, which skipped the
    /// outermost check too (depth is 1 at the first clause site); the test is
    /// now `depth > 1` (see the nested-tmp comment below).
    pub(crate) fn compile_contract_check(&mut self, expr: &Expr, clause_type: &str) {
        // Phase 5d: Emit contract expression as IR comment for debuggers
        let expr_str = format!("{:?}", expr).chars().take(80).collect::<String>();
        self.emitln(&format!("; contract: {clause_type}: {expr_str}"));
        let guarded = self.config.check_contracts && self.fn_has_contract_clauses();
        let ok_label = self.fresh_block("contract_ok");
        if guarded {
            // Branch to the ok (completion) label when re-entered. The
            // re-entry test is the function-entry `entry depth > 0` icmp
            // (m328: computed once by `emit_contract_guard_enter`); skip ->
            // ok -> continue; not-nested -> check -> ok -> continue. The
            // fallback keeps the per-site load+icmp shape for any clause
            // compiled without the entry emission.
            if let Some(nested) = self.fctx.contract_nested_tmp.clone() {
                let go_label = self.fresh_block("contract_reentry_go");
                self.emitln(&format!("  br i1 {nested}, label %{ok_label}, label %{go_label}"));
                self.emitln(&format!("\n{go_label}:"));
            } else if let Some(counter) = self.contract_counter_name() {
                self.declare_contract_counter(&counter);
                let d = self.fresh_tmp();
                self.emitln(&format!("  {d} = load i64, i64* @{counter}"));
                let nested = self.fresh_tmp();
                // m291 (REL-20261010-1548-stdlib-7): the counter counts
                // INVOCATIONS of this function on the stack (incremented at
                // entry, decremented on every return path). At the function's
                // own clause sites the OUTERMOST invocation has d == 1 and
                // must RUN; only a RE-ENTRANT evaluation (d > 1 -- the clause
                // called back into this function) skips. The m283 test was
                // `sgt d, 0`, which skipped EVERY check (d is already 1 at the
                // first clause): runtime requires/ensures enforcement was
                // silently dead for all clause-bearing functions (a violated
                // `requires: x > 0` with x=0 returned cleanly on HEAD and
                // aborted on v0.64.4).
                self.emitln(&format!("  {nested} = icmp sgt i64 {d}, 1"));
                let skip_label = self.fresh_block("contract_reentry_skip");
                let go_label = self.fresh_block("contract_reentry_go");
                self.emitln(&format!("  br i1 {nested}, label %{skip_label}, label %{go_label}"));
                self.emitln(&format!("\n{skip_label}:"));
                self.emitln(&format!("  br label %{ok_label}"));
                self.emitln(&format!("\n{go_label}:"));
            }
        }
        let (cond_val, expr_ty) = match self.compile_expr(expr) {
            Ok(v) => v,
            Err(_) => return,
        };
        let fail_label = self.fresh_block("contract_fail");
        // Ensure we have an i1 for the branch -- some expressions (or/and) return i64.
        // `expr_ty` is the value's real LLVM type as returned by compile_expr.
        let cond_i1 = if expr_ty == "i1" {
            cond_val
        } else if expr_ty == "i64" {
            let tmp = self.fresh_tmp();
            self.emitln(&format!("  {tmp} = icmp ne i64 {cond_val}, 0"));
            tmp
        } else {
            let ext = self.fresh_tmp();
            self.emitln(&format!("  {ext} = zext {expr_ty} {cond_val} to i64"));
            let tmp = self.fresh_tmp();
            self.emitln(&format!("  {tmp} = icmp ne i64 {ext}, 0"));
            tmp
        };
        // m311 (O2 no-fold barrier; COMPILER_BUGS m283 finding): clang -O2
        // constant-folds clause conditions whose operands become constant
        // after inlining (`requires: x != x`, `ensures: result == x + C` on a
        // constant input) and can then prune the check's violated-clause path
        // out of the optimized build -- enforcement dies only on optimized
        // builds. Route the final i1 through a volatile round-trip: LLVM must
        // not fold or elide volatile accesses, so the value stays
        // runtime-decided at every opt level and on every target (no
        // inline-asm dependency). m328: the round-trip is a direct i1
        // store/load through a per-site alloca (one byte in memory); the old
        // i8 spelling needed a zext + icmp pair at every clause site and was
        // the second half of the fmt_peek IR budget regression -- the volatile
        // guarantee is identical. The alloca sits at the clause site; a clause
        // check executes at most once per invocation (entry requires / return
        // ensures), so the slot cannot accumulate stack.
        let barrier_slot = self.fresh_tmp();
        self.emitln(&format!("  {barrier_slot} = alloca i1"));
        self.emitln(&format!("  store volatile i1 {cond_i1}, i1* {barrier_slot}"));
        let barrier_r = self.fresh_tmp();
        self.emitln(&format!("  {barrier_r} = load volatile i1, i1* {barrier_slot}"));
        self.emitln(&format!("  br i1 {barrier_r}, label %{ok_label}, label %{fail_label}"));
        self.emitln(&format!("\n{fail_label}:"));
        let msg_ptr = self.fresh_tmp();
        let msg = format!("contract violated: {clause_type} at {}", expr.span());
        let str_id = self.str_counter;
        self.str_counter += 1;
        let label = format!("@.contract_str{str_id}");
        let escaped = msg.replace('\\', "\\\\").replace('"', "\\22");
        self.fctx.strings.push(format!(
            "{label} = private unnamed_addr constant [{len} x i8] c\"{escaped}\\00\"",
            len = msg.len() + 1
        ));
        self.emitln(&format!("  {msg_ptr} = getelementptr [{len} x i8], [{len} x i8]* {label}, i64 0, i64 0",
            len = msg.len() + 1));
        self.emitln(&format!("  call i32 @puts(i8* {msg_ptr})"));
        // BUG 22 #5 fix (2026-08-12): a contract violation is a LOGIC error --
        // fail FAST with the message visible and a clean exit code, not a
        // hardware trap. The old `llvm.trap()` (ud2 -> 0xC000001D) crashed the
        // process with any buffered output lost, and inside unsafe blocks the
        // VEH caught the trap as a recoverable "illegal instruction" fault,
        // silently swallowing real contract violations.
        self.emitln(&format!("  call void @xiom_panic(i8* {msg_ptr})"));
        self.emitln("  unreachable");
        self.emitln(&format!("\n{ok_label}:"));
    }

    /// Emit checks for all ensures clauses of the current function.
    /// Called just before a return instruction.
    pub(crate) fn compile_ensures_checks(&mut self) {
        for expr in &self.fctx.current_ensures.clone() {
            // BUG 30: bind `result` to the RETURN-VALUE slot in a dedicated
            // scope for the duration of each check. Without it, a USER local
            // named `result` (e.g. `var result = Vec[UInt8].new()` inside
            // utf8_encode) shadows the synthetic return-slot binding at later
            // return sites, and the ensure `result is Ok => result.len()`
            // reads the user's Vec header as the Result struct (AV).
            self.push_scope();
            // 2026-10-05 (packages-lane relay): `local_xiom_types` and
            // `is_payload_rebind` are FLAT maps -- `push_scope`/`pop_scope`
            // only scope LLVM slots -- so contract-expression compilation
            // leaked binding state across clauses (and back into the body):
            //   * a body local named `result` (varint_decode_u: `var result:
            //     Int = 0;`) overwrote the declared return type recorded at
            //     function setup;
            //   * a previous clause's bare `is Err/Ok` payload rebind
            //     (`bind_is_payload_xiom`) overwrote it (cobs_decode has the
            //     Err clause before the Ok payload bound).
            // The poisoned type then lowered `result.value.1` to literal 0
            // and `result.value.len()` to `xiom_str_len` on a boxed Vec
            // payload handle -- spurious `contract violated: ensures` at
            // runtime for structurally true clauses. Snapshot the binding
            // state around each clause, start it from the DECLARED return
            // type, and restore afterwards.
            let saved_xiom_types = self.local.local_xiom_types.clone();
            let saved_payload_rebinds = self.local.is_payload_rebind.clone();
            let saved_vec_elem = self.local.local_vec_elem.clone();
            let saved_vec_handle = self.local.local_vec_handle.clone();
            if let (Some(ptr), Some(ty)) = (self.fctx.result_ptr.clone(), self.fctx.result_llvm_ty.clone()) {
                self.add_local("result", ptr, &ty);
                if let Some(rt) = self.fctx.result_xiom_ty.clone() {
                    self.local.local_xiom_types.insert("result".to_string(), rt.clone());
                    self.local.is_payload_rebind.remove("result");
                    // m301 (REL-20261010-1549-stdlib-4): record the return Vec's
                    // ELEMENT type for the synthetic `result` binding, exactly as
                    // a `var v = <call>` binding does (stmt.rs inheritance from
                    // `callee_return_xiom`). Without it, `result[0]` in clause
                    // position could not resolve the element and fell to the
                    // scalar `emit_elem_load` fallback, which loads the raw i64
                    // slot: for Vec[Float64] the f64 BIT PATTERN was sitofp'd
                    // (1.0 -> 4.6e18 -> false `contract violated`); for
                    // Vec[Vec[T]] the inner Vec's data pointer was loaded and
                    // inttoptr'd as a Vec header, so `result[0].len()` read the
                    // inner ARRAY as the row (the Int control only passed when
                    // the row's second element happened to equal the expected
                    // length). The ensures check restores this flat map after
                    // every clause like local_xiom_types.
                    if let Some(elem) = rt.strip_prefix("Vec[").and_then(|r| r.strip_suffix(']')) {
                        self.local.local_vec_elem.insert("result".to_string(), elem.to_string());
                    }
                }
            }
            self.compile_contract_check(expr, "ensures");
            self.local.local_xiom_types = saved_xiom_types;
            self.local.is_payload_rebind = saved_payload_rebinds;
            self.local.local_vec_elem = saved_vec_elem;
            self.local.local_vec_handle = saved_vec_handle;
            self.pop_scope();
        }
    }

    /// Generate an invariant check function for a struct type.
    pub(crate) fn compile_invariant_check(&mut self, type_name: &str) -> Result<(), String> {
        let invariants = match self.types.type_meta.get(&type_name.to_string()) {
            Some(m) => m.invariants.clone(),
            None => return Ok(()),
        };
        if invariants.is_empty() {
            return Ok(());
        }
        // Look up field names for this type
        let field_names = match self.types.types.get(&type_name.to_string()) {
            Some(f) => f.clone(),
            None => return Ok(()),
        };
        let struct_ty = format!("%struct.{type_name}");
        let fn_name = format!("{type_name}.invariant_check");
        // m326 (REL-20261014-2123-stdlib): emit-once guard, mirroring every
        // other derived-impl emitter (compile_eq_impl/clone/display/hash/...).
        // Without it a program carrying two TypeDecl nodes for the same type
        // (e.g. an injected external decl pushed twice) emitted the helper as
        // a duplicate symbol and clang rejected the module ("invalid
        // redefinition of function 'LinkedList.invariant_check'").
        if self.mono.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.mono.emitted_fns.insert(fn_name.clone());
        self.emitln(&format!("define void @{fn_name}({struct_ty} %__obj) {{"));
        // Store the struct value in an alloca for field access
        let alloca = self.fresh_tmp();
        self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %__obj, {struct_ty}* {alloca}"));
        // Register field locals for invariant expression compilation
        for (i, fname) in field_names.iter().enumerate() {
            let gep = self.fresh_tmp();
            let loaded = self.fresh_tmp();
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
            // Create an alloca so compile_expr can load from it via the pointer
            let field_alloca = self.fresh_tmp();
            self.emitln(&format!("  {field_alloca} = alloca {field_llvm_ty}"));
            self.emitln(&format!("  store {field_llvm_ty} {loaded}, {field_llvm_ty}* {field_alloca}"));
            // Push a synthetic scope for invariant compilation
            // Since compile_contract_check will call compile_expr which uses lookup_local,
            // we need to register these field names temporarily
            if self.fctx.locals.is_empty() {
                self.fctx.locals.push(HashMap::new());
            }
            self.add_local(fname, field_alloca, &field_llvm_ty);
        }
        for inv in &invariants {
            self.compile_contract_check(inv, "invariant");
        }
        self.emitln("  ret void");
        self.emitln("}\n");
        // Clean up the temporary field locals
        // We added them to the current scope, they'll be cleaned on pop_scope
        // But since we're not actually pushing a real scope, let's just clear
        if let Some(scope) = self.fctx.locals.last_mut() {
            for fname in &field_names {
                scope.remove(fname);
            }
        }
        Ok(())
    }

    /// Emit a call to a type's invariant check function.
    /// Helper: given an expression and its compiled value register, emit invariant
    /// check if the expression evaluates to a struct type that has invariants.
    pub(crate) fn maybe_check_value_invariants(&mut self, value: &Expr, val_reg: &str) {
        let type_name = self.struct_type_from_expr(value);
        if let Some(ref tn) = type_name {
            if self.types.type_meta.get(&tn.to_string()).map(|m| !m.invariants.is_empty()).unwrap_or(false) {
                self.compile_invariant_call(tn, val_reg);
            }
        }
    }

    /// Store a by-value struct `val` (LLVM type `ty`) back into the storage of
    /// `receiver` so in-place mutation (`Vec.push`/`Vec.pop`) persists the result.
    /// Handles bare locals AND struct field access (e.g. `h.entries.push(...)`).
    pub(crate) fn store_back_to_receiver(&mut self, receiver: &Expr, val: &str, ty: &str) {
        // Case 1: bare local variable -- store to its alloca slot.
        if let Expr::Ident(id) = receiver {
            if let Some((slot, slot_ty)) = self.lookup_local(&id.name).cloned() {
                if slot_ty == ty {
                    self.emitln(&format!("  store {ty} {val}, {ty}* {slot}"));
                    return;
                }
                if slot_ty.ends_with('*') {
                    let inner = slot_ty.trim_end_matches('*');
                    if inner == ty {
                        let ptr_val = self.fresh_tmp();
                        self.emitln(&format!("  {ptr_val} = load {slot_ty}, {slot_ty}* {slot}"));
                        self.emitln(&format!("  store {ty} {val}, {ty}* {ptr_val}"));
                        return;
                    }
                }
                // 5c.30: i64 container-handle local (match-arm payload binding):
                // write the updated header THROUGH the boxed pointer so the
                // mutation aliases the original enum/struct field.
                if slot_ty == "i64"
                    && ty.starts_with("%struct.")
                    && self.local.local_vec_handle.contains_key(&id.name)
                {
                    let h = self.fresh_tmp();
                    self.emitln(&format!("  {h} = load i64, i64* {slot}"));
                    let boxp = self.fresh_tmp();
                    self.emitln(&format!("  {boxp} = inttoptr i64 {h} to {ty}*"));
                    self.emitln(&format!("  store {ty} {val}, {ty}* {boxp}"));
                    return;
                }
            }
        }
        // Case 3: Index expression (e.g. outer[0].push(x)) -- store the
        // modified struct back to the element position in the buffer.
        if let Expr::Index(container, idx, _) = receiver {
            if let Some(elem_ptr) = self.resolve_index_elem_ptr(container, idx) {
                let vp = self.fresh_tmp();
                self.emitln(&format!("  {vp} = bitcast i8* {elem_ptr} to {ty}*"));
                self.emitln(&format!("  store {ty} {val}, {ty}* {vp}"));
                return;
            }
        }
        // Case 2: struct field access -- GEP into the base struct and store.
        if let Expr::Field(base, field_expr, _) = receiver {
            if let Expr::Ident(base_id) = &**base {
                if let Some((slot, slot_ty)) = self.lookup_local(&base_id.name).cloned() {
                    // Resolve the struct type name from the base's declared type.
                    let sty = if slot_ty.starts_with("%struct.") {
                        slot_ty.clone()
                    } else {
                        return; // cannot determine struct type
                    };
                    let clean_name = sty[8..].trim_end_matches('*').to_string();
                    if let Some(field_names) = self.types.types.get(&clean_name)
                        .or_else(|| self.types.types.keys().into_iter()
    .find(|k| k.ends_with(&format!(".{clean_name}")))
                            .and_then(|k|self.types.types.get(&k)))
                        
                    {
                        if let Some(fi) = field_names.iter().position(|f| f == &field_expr.name) {
                            let sty_clean = format!("%struct.{clean_name}");
                            // Compute the field pointer (base may be &mut T or by-value).
                            let gep = if slot_ty.ends_with('*') {
                                // Base is a pointer (&mut T): load the pointer, then GEP.
                                let ptr = self.fresh_tmp();
                                self.emitln(&format!("  {ptr} = load {slot_ty}, {slot_ty}* {slot}"));
                                let gep = self.fresh_tmp();
                                self.emitln(&format!("  {gep} = getelementptr {sty_clean}, {sty_clean}* {ptr}, i32 0, i32 {fi}"));
                                gep
                            } else {
                                // Base is by-value: GEP directly into the alloca.
                                let gep = self.fresh_tmp();
                                self.emitln(&format!("  {gep} = getelementptr {sty_clean}, {sty_clean}* {slot}, i32 0, i32 {fi}"));
                                gep
                            };
                            // 5c.29: Generic container fields are i64 HANDLES
                            // (5c.28h): the slot holds a pointer to a heap-boxed
                            // header. Store the updated header THROUGH the handle
                            // (in-place box update) -- never the 32-byte header
                            // by value into the 8-byte slot.
                            let is_handle_field = ty.starts_with("%struct.")
                                && !ty.ends_with('*')
                                && self.field_llvm_type(&clean_name, fi) == "i64"
                                && self.field_xiom_type(&clean_name, fi)
                                    .map_or(false, |t| t.contains('['));
                            if is_handle_field {
                                let handle = self.fresh_tmp();
                                self.emitln(&format!("  {handle} = load i64, i64* {gep}"));
                                let boxp = self.fresh_tmp();
                                self.emitln(&format!("  {boxp} = inttoptr i64 {handle} to {ty}*"));
                                self.emitln(&format!("  store {ty} {val}, {ty}* {boxp}"));
                            } else {
                                self.emitln(&format!("  store {ty} {val}, {ty}* {gep}"));
                            }
                        }
                    }
                }
            }
            // M33: Compound field access through &mut (e.g. tree.nodes[idx].keys).
            // When the base of a field access is NOT a simple Ident (it's an Index,
            // another Field, or a chain), use compile_lvalue to compute the GEP
            // pointer. This handles `tree.nodes[idx].keys.insert(key)` where the
            // receiver spans a Vec index + struct field through a &mut reference.
            if let Some((l_ptr, l_ptr_ty, _l_elem_ty)) = self.compile_lvalue(receiver) {
                if l_ptr_ty.ends_with('*') && ty == l_ptr_ty.trim_end_matches('*') {
                    self.emitln(&format!("  store {ty} {val}, {l_ptr_ty} {l_ptr}"));
                    return;
                }
                // Handle pointer-type match: the lvalue gave us a pointer,
                // store through it.
                if l_ptr_ty.ends_with('*') {
                    self.emitln(&format!("  store {ty} {val}, {l_ptr_ty} {l_ptr}"));
                    return;
                }
            }
        }
    }

    pub(crate) fn compile_invariant_call(&mut self, type_name: &str, struct_val_reg: &str) {
        let meta = match self.types.type_meta.get(&type_name.to_string()) {
            Some(m) => m,
            None => return,
        };
        if meta.invariants.is_empty() {
            return;
        }
        let fn_name = format!("{type_name}.invariant_check");
        let struct_ty = format!("%struct.{type_name}");
        self.emitln(&format!("  call void @{fn_name}({struct_ty} {struct_val_reg})"));
    }
}
