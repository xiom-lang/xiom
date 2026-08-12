use std::collections::HashMap;
use xiom_ast::*;

impl super::IrEmitter {
    /// Emit a runtime check for a single boolean contract expression.
    /// If the expression evaluates to false (i64 0), emit a panic and trap.
    pub(crate) fn compile_contract_check(&mut self, expr: &Expr, clause_type: &str) {
        // Phase 5d: Emit contract expression as IR comment for debuggers
        let expr_str = format!("{:?}", expr).chars().take(80).collect::<String>();
        self.emitln(&format!("; contract: {clause_type}: {expr_str}"));
        let (cond_val, expr_ty) = match self.compile_expr(expr) {
            Ok(v) => v,
            Err(_) => return,
        };
        let ok_label = self.fresh_block("contract_ok");
        let fail_label = self.fresh_block("contract_fail");
        // Ensure we have an i1 for the branch — some expressions (or/and) return i64.
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
        self.emitln(&format!("  br i1 {cond_i1}, label %{ok_label}, label %{fail_label}"));
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
        // BUG 22 #5 fix (2026-08-12): a contract violation is a LOGIC error —
        // fail FAST with the message visible and a clean exit code, not a
        // hardware trap. The old `llvm.trap()` (ud2 → 0xC000001D) crashed the
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
            self.compile_contract_check(expr, "ensures");
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
        // Case 1: bare local variable — store to its alloca slot.
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
        // Case 3: Index expression (e.g. outer[0].push(x)) — store the
        // modified struct back to the element position in the buffer.
        if let Expr::Index(container, idx, _) = receiver {
            if let Some(elem_ptr) = self.resolve_index_elem_ptr(container, idx) {
                let vp = self.fresh_tmp();
                self.emitln(&format!("  {vp} = bitcast i8* {elem_ptr} to {ty}*"));
                self.emitln(&format!("  store {ty} {val}, {ty}* {vp}"));
                return;
            }
        }
        // Case 2: struct field access — GEP into the base struct and store.
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
                            // (in-place box update) — never the 32-byte header
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
