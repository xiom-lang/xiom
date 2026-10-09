// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: Apache-2.0

//! m239 (queue item 20): deep CONTENT equality for container values.
//!
//! `==`/`!=` on `%struct.Vec` and the erased Option/Result shells used to
//! compare the erased i64 field bits (data pointers / boxed payload
//! handles): equal values built separately compared FALSE, and
//! `Ok(Vec) == Ok(Vec)` was silently wrong. This module emits a recursive
//! content comparison:
//!
//! - `Vec[T]`: length equality + elementwise recursion through an LLVM GEP
//!   stride (padding-aware, mirroring the storage path from m234).
//! - `Str`: strcmp content compare (same lowering as the scalar `==`).
//! - `Option[T]` / `Result[T, E]`: tag equality, then recursion into the
//!   ACTIVE payload only (boxed aggregates are unboxed; Str/raw scalars are
//!   compared in the erased slot). Inactive payload slots are never read.
//! - User structs reached as elements/payloads: derived `.eq` when
//!   registered, else field-by-field recursion over `type_meta`.
//!
//! Map/Set are deliberately NOT engaged (lookup-based equality needs its
//! own design pass -- COMPILER_BUGS 2026-10-09 queue item 20). Recursive
//! payload types (linked lists / trees) hit a repetition guard at the
//! boundary and fall back to handle identity instead of looping forever.

use super::IrEmitter;
use xiom_ast::Expr;

/// m239: depth cap for pathological nesting; the repetition guard normally
/// fires first.
const DEEP_EQ_MAX_DEPTH: u32 = 12;

impl IrEmitter {
    /// m239: XIOM type hint for an `==` operand -- tracked local types
    /// first, literal/ctor/call-return inference otherwise. None keeps the
    /// historical lowering.
    pub(crate) fn operand_xiom_hint(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(id) => self.xiom_type_of_local(&id.name),
            Expr::Paren(inner, _) => self.operand_xiom_hint(inner),
            _ => Self::infer_value_xiom_type(expr).or_else(|| self.infer_call_return_xiom(expr)),
        }
    }

    /// m239: is `ty` a container whose `==` must compare CONTENT? Map/Set
    /// are excluded by design (lookup-based, design pending).
    pub(crate) fn is_deep_eq_container(ty: &str) -> bool {
        let (base, _) = Self::deep_split_type(ty);
        matches!(base.as_str(), "Vec" | "Option" | "Result")
    }

    /// m239: the operand's LLVM value must be the matching by-value struct
    /// shell (erased `%struct.Vec` or a concrete `%struct.Option__X`). Raw
    /// pointers and degraded scalars must keep the old lowering -- emitting
    /// `store %struct.Vec <i64>` would be invalid IR.
    pub(crate) fn deep_eq_llvm_compatible(ty: &str, llvm: &str) -> bool {
        if !llvm.starts_with("%struct.") || llvm.ends_with('*') {
            return false;
        }
        let (base, _) = Self::deep_split_type(ty);
        match base.as_str() {
            "Vec" => llvm == "%struct.Vec" || llvm.contains("Vec"),
            "Option" => llvm.contains("Option"),
            "Result" => llvm.contains("Result"),
            _ => false,
        }
    }

    /// m239: emit a content comparison for two values of `ty`, returning an
    /// i64 register holding 0/1. Entry point for the `==`/`!=` lowering.
    pub(crate) fn emit_deep_eq_operands(
        &mut self,
        ty: &str,
        l_val: &str,
        l_llvm: &str,
        r_val: &str,
        r_llvm: &str,
    ) -> String {
        let mut visited: Vec<String> = Vec::new();
        self.emit_deep_eq(ty, l_val, l_llvm, r_val, r_llvm, &mut visited, 0)
    }

    /// Depth/parens-aware split of a generic type spelling:
    /// "Result[Vec[UInt8], Int]" -> ("Result", ["Vec[UInt8]", "Int"]).
    /// Unlike `parse_generic_type_string`, parentheses nest too (tuples).
    fn deep_split_type(s: &str) -> (String, Vec<String>) {
        let Some(open) = s.find('[') else {
            return (s.trim().to_string(), Vec::new());
        };
        let Some(close) = s.rfind(']') else {
            return (s.trim().to_string(), Vec::new());
        };
        let base = s[..open].trim().to_string();
        let inner = &s[open + 1..close];
        let mut args: Vec<String> = Vec::new();
        let mut depth = 0i32;
        let mut current = String::new();
        for c in inner.chars() {
            match c {
                '[' | '(' => {
                    depth += 1;
                    current.push(c);
                }
                ']' | ')' => {
                    depth -= 1;
                    current.push(c);
                }
                ',' if depth == 0 => {
                    args.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        if !current.trim().is_empty() {
            args.push(current.trim().to_string());
        }
        (base, args)
    }

    fn emit_deep_eq(
        &mut self,
        ty: &str,
        l_val: &str,
        l_llvm: &str,
        r_val: &str,
        r_llvm: &str,
        visited: &mut Vec<String>,
        depth: u32,
    ) -> String {
        if depth > DEEP_EQ_MAX_DEPTH || visited.iter().any(|t| t == ty) {
            // Recursive payload/element type (or pathological nesting):
            // bounded fallback -- compare the value's lead scalar/handle
            // (identity for buffers/handles). Documented limitation.
            return self.emit_identity_fallback(l_llvm, l_val, r_val);
        }
        let (base, args) = Self::deep_split_type(ty);
        visited.push(ty.to_string());
        let out = match base.as_str() {
            "Vec" => self.emit_deep_vec_eq(
                args.first().map(String::as_str).unwrap_or("Int"),
                l_val,
                r_val,
                visited,
                depth,
            ),
            "Str" => self.emit_str_content_eq(l_val, l_llvm, r_val, r_llvm),
            "Option" => {
                if l_llvm.starts_with("%struct.") && !l_llvm.contains("__") {
                    self.emit_deep_option_eq(
                        args.first().map(String::as_str).unwrap_or("Int"),
                        l_val,
                        r_val,
                        visited,
                        depth,
                    )
                } else {
                    // Concrete shell (Option__X): typed fields, no boxed
                    // erase -- the aggregate comparator handles it.
                    self.emit_deep_aggregate_eq(l_val, l_llvm, r_val, r_llvm, visited, depth)
                }
            }
            "Result" => {
                if l_llvm.starts_with("%struct.") && !l_llvm.contains("__") {
                    self.emit_deep_result_eq(
                        args.first().map(String::as_str).unwrap_or("Int"),
                        args.get(1).map(String::as_str).unwrap_or("Int"),
                        l_val,
                        r_val,
                        visited,
                        depth,
                    )
                } else {
                    self.emit_deep_aggregate_eq(l_val, l_llvm, r_val, r_llvm, visited, depth)
                }
            }
            _ => self.emit_deep_aggregate_eq(l_val, l_llvm, r_val, r_llvm, visited, depth),
        };
        visited.pop();
        out
    }

    /// Str content equality via strcmp (parity with the scalar `==` path).
    fn emit_str_content_eq(&mut self, l_val: &str, l_llvm: &str, r_val: &str, r_llvm: &str) -> String {
        let lp = self.val_to_i8ptr(l_val, l_llvm);
        let rp = self.val_to_i8ptr(r_val, r_llvm);
        let cmp = self.fresh_tmp();
        self.emitln(&format!("  {cmp} = call i32 @strcmp(i8* {lp}, i8* {rp})"));
        let is_eq = self.fresh_tmp();
        self.emitln(&format!("  {is_eq} = icmp eq i32 {cmp}, 0"));
        let ext = self.fresh_tmp();
        self.emitln(&format!("  {ext} = zext i1 {is_eq} to i64"));
        ext
    }

    /// Scalar equality for one element/payload pair, dispatching on the
    /// LLVM spelling (float vs integer).
    fn emit_scalar_eq(&mut self, l_val: &str, l_llvm: &str, r_val: &str, _r_llvm: &str) -> String {
        let (cmp, ext) = (self.fresh_tmp(), self.fresh_tmp());
        if matches!(l_llvm, "double" | "float" | "fp128") {
            self.emitln(&format!("  {cmp} = fcmp oeq {l_llvm} {l_val}, {r_val}"));
        } else if l_llvm == "i1" {
            self.emitln(&format!("  {cmp} = icmp eq i1 {l_val}, {r_val}"));
        } else {
            self.emitln(&format!("  {cmp} = icmp eq {l_llvm} {l_val}, {r_val}"));
        }
        self.emitln(&format!("  {ext} = zext i1 {cmp} to i64"));
        ext
    }

    /// Bounded fallback for recursive types: compare the lead scalar of each
    /// value (Vec data pointer / Option payload / first i64 field; scalar
    /// bit patterns otherwise). Terminates; equal-but-distinct cyclic
    /// structures compare false (documented limitation).
    fn emit_identity_fallback(&mut self, l_llvm: &str, l_val: &str, r_val: &str) -> String {
        if l_llvm.starts_with("%struct.") && !l_llvm.ends_with('*') {
            let li = self.extract_scalar_field0(l_val, l_llvm);
            let ri = self.extract_scalar_field0(r_val, l_llvm);
            let (cmp, ext) = (self.fresh_tmp(), self.fresh_tmp());
            self.emitln(&format!("  {cmp} = icmp eq i64 {li}, {ri}"));
            self.emitln(&format!("  {ext} = zext i1 {cmp} to i64"));
            ext
        } else {
            self.emit_scalar_eq(l_val, l_llvm, r_val, l_llvm)
        }
    }

    /// `%struct.Vec` content equality: length check, then elementwise
    /// recursion. Elements are addressed through an LLVM GEP on the element
    /// type, so the stride matches the padded storage (m234).
    fn emit_deep_vec_eq(
        &mut self,
        elem_xiom: &str,
        l_val: &str,
        r_val: &str,
        visited: &mut Vec<String>,
        depth: u32,
    ) -> String {
        const VTY: &str = "%struct.Vec";
        let elem_llvm = self.field_llvm_ty(elem_xiom);
        let l_a = self.fresh_tmp();
        self.emitln(&format!("  {l_a} = alloca {VTY}"));
        self.emitln(&format!("  store {VTY} {l_val}, {VTY}* {l_a}"));
        let r_a = self.fresh_tmp();
        self.emitln(&format!("  {r_a} = alloca {VTY}"));
        self.emitln(&format!("  store {VTY} {r_val}, {VTY}* {r_a}"));
        let idx_a = self.fresh_tmp();
        self.emitln(&format!("  {idx_a} = alloca i64"));
        self.emitln(&format!("  store i64 0, i64* {idx_a}"));
        let lg = self.fresh_tmp();
        self.emitln(&format!("  {lg} = getelementptr {VTY}, {VTY}* {l_a}, i32 0, i32 1"));
        let len_l = self.fresh_tmp();
        self.emitln(&format!("  {len_l} = load i64, i64* {lg}"));
        let rg = self.fresh_tmp();
        self.emitln(&format!("  {rg} = getelementptr {VTY}, {VTY}* {r_a}, i32 0, i32 1"));
        let len_r = self.fresh_tmp();
        self.emitln(&format!("  {len_r} = load i64, i64* {rg}"));
        let len_eq = self.fresh_tmp();
        self.emitln(&format!("  {len_eq} = icmp eq i64 {len_l}, {len_r}"));

        let ne_b = self.fresh_block("deq_vec_ne");
        let len_b = self.fresh_block("deq_vec_len");
        let cond_b = self.fresh_block("deq_vec_cond");
        let body_b = self.fresh_block("deq_vec_body");
        let next_b = self.fresh_block("deq_vec_next");
        let bad_b = self.fresh_block("deq_vec_bad");
        let end_b = self.fresh_block("deq_vec_end");
        let merge_b = self.fresh_block("deq_vec_merge");

        self.emitln(&format!("  br i1 {len_eq}, label %{len_b}, label %{ne_b}"));
        self.emitln(&format!("\n{ne_b}:"));
        self.emitln(&format!("  br label %{merge_b}"));
        self.emitln(&format!("\n{len_b}:"));
        self.emitln(&format!("  br label %{cond_b}"));
        self.emitln(&format!("\n{cond_b}:"));
        let iv0 = self.fresh_tmp();
        self.emitln(&format!("  {iv0} = load i64, i64* {idx_a}"));
        let cont = self.fresh_tmp();
        self.emitln(&format!("  {cont} = icmp slt i64 {iv0}, {len_l}"));
        self.emitln(&format!("  br i1 {cont}, label %{body_b}, label %{end_b}"));
        self.emitln(&format!("\n{body_b}:"));
        let ld = self.fresh_tmp();
        self.emitln(&format!("  {ld} = getelementptr {VTY}, {VTY}* {l_a}, i32 0, i32 0"));
        let lp = self.fresh_tmp();
        self.emitln(&format!("  {lp} = load i8*, i8** {ld}"));
        let lpe = self.fresh_tmp();
        self.emitln(&format!("  {lpe} = bitcast i8* {lp} to {elem_llvm}*"));
        let le = self.fresh_tmp();
        self.emitln(&format!("  {le} = getelementptr {elem_llvm}, {elem_llvm}* {lpe}, i64 {iv0}"));
        let lv = self.fresh_tmp();
        self.emitln(&format!("  {lv} = load {elem_llvm}, {elem_llvm}* {le}"));
        let rd = self.fresh_tmp();
        self.emitln(&format!("  {rd} = getelementptr {VTY}, {VTY}* {r_a}, i32 0, i32 0"));
        let rp = self.fresh_tmp();
        self.emitln(&format!("  {rp} = load i8*, i8** {rd}"));
        let rpe = self.fresh_tmp();
        self.emitln(&format!("  {rpe} = bitcast i8* {rp} to {elem_llvm}*"));
        let re = self.fresh_tmp();
        self.emitln(&format!("  {re} = getelementptr {elem_llvm}, {elem_llvm}* {rpe}, i64 {iv0}"));
        let rv = self.fresh_tmp();
        self.emitln(&format!("  {rv} = load {elem_llvm}, {elem_llvm}* {re}"));
        let peq = self.emit_deep_eq(elem_xiom, &lv, &elem_llvm, &rv, &elem_llvm, visited, depth + 1);
        let pi = self.fresh_tmp();
        self.emitln(&format!("  {pi} = icmp ne i64 {peq}, 0"));
        self.emitln(&format!("  br i1 {pi}, label %{next_b}, label %{bad_b}"));
        self.emitln(&format!("\n{next_b}:"));
        let ni = self.fresh_tmp();
        self.emitln(&format!("  {ni} = add i64 {iv0}, 1"));
        self.emitln(&format!("  store i64 {ni}, i64* {idx_a}"));
        self.emitln(&format!("  br label %{cond_b}"));
        self.emitln(&format!("\n{bad_b}:"));
        self.emitln(&format!("  br label %{merge_b}"));
        self.emitln(&format!("\n{end_b}:"));
        self.emitln(&format!("  br label %{merge_b}"));
        self.emitln(&format!("\n{merge_b}:"));
        let phi = self.fresh_tmp();
        self.emitln(&format!(
            "  {phi} = phi i64 [ 0, %{ne_b} ], [ 0, %{bad_b} ], [ 1, %{end_b} ]"
        ));
        phi
    }

    /// Erased `%struct.Option`: tag equality + ACTIVE payload compare.
    fn emit_deep_option_eq(
        &mut self,
        payload_xiom: &str,
        l_val: &str,
        r_val: &str,
        visited: &mut Vec<String>,
        depth: u32,
    ) -> String {
        const OTY: &str = "%struct.Option";
        let l_a = self.fresh_tmp();
        self.emitln(&format!("  {l_a} = alloca {OTY}"));
        self.emitln(&format!("  store {OTY} {l_val}, {OTY}* {l_a}"));
        let r_a = self.fresh_tmp();
        self.emitln(&format!("  {r_a} = alloca {OTY}"));
        self.emitln(&format!("  store {OTY} {r_val}, {OTY}* {r_a}"));
        let lg = self.fresh_tmp();
        self.emitln(&format!("  {lg} = getelementptr {OTY}, {OTY}* {l_a}, i32 0, i32 0"));
        let lt = self.fresh_tmp();
        self.emitln(&format!("  {lt} = load i64, i64* {lg}"));
        let rg = self.fresh_tmp();
        self.emitln(&format!("  {rg} = getelementptr {OTY}, {OTY}* {r_a}, i32 0, i32 0"));
        let rt = self.fresh_tmp();
        self.emitln(&format!("  {rt} = load i64, i64* {rg}"));
        let teq = self.fresh_tmp();
        self.emitln(&format!("  {teq} = icmp eq i64 {lt}, {rt}"));
        let is_some = self.fresh_tmp();
        self.emitln(&format!("  {is_some} = icmp eq i64 {lt}, 1"));

        let ne_b = self.fresh_block("deq_opt_ne");
        let cont_b = self.fresh_block("deq_opt_cont");
        let some_b = self.fresh_block("deq_opt_some");
        let none_b = self.fresh_block("deq_opt_none");
        let ok_b = self.fresh_block("deq_opt_ok");
        let bad_b = self.fresh_block("deq_opt_bad");
        let merge_b = self.fresh_block("deq_opt_merge");

        self.emitln(&format!("  br i1 {teq}, label %{cont_b}, label %{ne_b}"));
        self.emitln(&format!("\n{ne_b}:"));
        self.emitln(&format!("  br label %{merge_b}"));
        self.emitln(&format!("\n{cont_b}:"));
        self.emitln(&format!("  br i1 {is_some}, label %{some_b}, label %{none_b}"));
        self.emitln(&format!("\n{some_b}:"));
        let lpg = self.fresh_tmp();
        self.emitln(&format!("  {lpg} = getelementptr {OTY}, {OTY}* {l_a}, i32 0, i32 1"));
        let lp = self.fresh_tmp();
        self.emitln(&format!("  {lp} = load i64, i64* {lpg}"));
        let rpg = self.fresh_tmp();
        self.emitln(&format!("  {rpg} = getelementptr {OTY}, {OTY}* {r_a}, i32 0, i32 1"));
        let rp = self.fresh_tmp();
        self.emitln(&format!("  {rp} = load i64, i64* {rpg}"));
        let peq = self.emit_payload_eq(payload_xiom, &lp, &rp, visited, depth);
        let pi = self.fresh_tmp();
        self.emitln(&format!("  {pi} = icmp ne i64 {peq}, 0"));
        self.emitln(&format!("  br i1 {pi}, label %{ok_b}, label %{bad_b}"));
        self.emitln(&format!("\n{none_b}:"));
        self.emitln(&format!("  br label %{merge_b}"));
        self.emitln(&format!("\n{ok_b}:"));
        self.emitln(&format!("  br label %{merge_b}"));
        self.emitln(&format!("\n{bad_b}:"));
        self.emitln(&format!("  br label %{merge_b}"));
        self.emitln(&format!("\n{merge_b}:"));
        let phi = self.fresh_tmp();
        self.emitln(&format!(
            "  {phi} = phi i64 [ 0, %{ne_b} ], [ 1, %{none_b} ], [ 1, %{ok_b} ], [ 0, %{bad_b} ]"
        ));
        phi
    }

    /// Erased `%struct.Result`: tag equality + ACTIVE payload compare
    /// (Ok = tag 1 -> field 1, Err = tag 0 -> field 2).
    fn emit_deep_result_eq(
        &mut self,
        ok_xiom: &str,
        err_xiom: &str,
        l_val: &str,
        r_val: &str,
        visited: &mut Vec<String>,
        depth: u32,
    ) -> String {
        const RTY: &str = "%struct.Result";
        let l_a = self.fresh_tmp();
        self.emitln(&format!("  {l_a} = alloca {RTY}"));
        self.emitln(&format!("  store {RTY} {l_val}, {RTY}* {l_a}"));
        let r_a = self.fresh_tmp();
        self.emitln(&format!("  {r_a} = alloca {RTY}"));
        self.emitln(&format!("  store {RTY} {r_val}, {RTY}* {r_a}"));
        let lg = self.fresh_tmp();
        self.emitln(&format!("  {lg} = getelementptr {RTY}, {RTY}* {l_a}, i32 0, i32 0"));
        let lt = self.fresh_tmp();
        self.emitln(&format!("  {lt} = load i64, i64* {lg}"));
        let rg = self.fresh_tmp();
        self.emitln(&format!("  {rg} = getelementptr {RTY}, {RTY}* {r_a}, i32 0, i32 0"));
        let rt = self.fresh_tmp();
        self.emitln(&format!("  {rt} = load i64, i64* {rg}"));
        let teq = self.fresh_tmp();
        self.emitln(&format!("  {teq} = icmp eq i64 {lt}, {rt}"));
        let is_ok = self.fresh_tmp();
        self.emitln(&format!("  {is_ok} = icmp eq i64 {lt}, 1"));

        let ne_b = self.fresh_block("deq_res_ne");
        let cont_b = self.fresh_block("deq_res_cont");
        let ok_b = self.fresh_block("deq_res_ok");
        let err_b = self.fresh_block("deq_res_err");
        let okok_b = self.fresh_block("deq_res_okok");
        let okbad_b = self.fresh_block("deq_res_okbad");
        let errok_b = self.fresh_block("deq_res_errok");
        let errbad_b = self.fresh_block("deq_res_errbad");
        let merge_b = self.fresh_block("deq_res_merge");

        self.emitln(&format!("  br i1 {teq}, label %{cont_b}, label %{ne_b}"));
        self.emitln(&format!("\n{ne_b}:"));
        self.emitln(&format!("  br label %{merge_b}"));
        self.emitln(&format!("\n{cont_b}:"));
        self.emitln(&format!("  br i1 {is_ok}, label %{ok_b}, label %{err_b}"));
        // Ok payload (field 1).
        self.emitln(&format!("\n{ok_b}:"));
        let lpg = self.fresh_tmp();
        self.emitln(&format!("  {lpg} = getelementptr {RTY}, {RTY}* {l_a}, i32 0, i32 1"));
        let lp = self.fresh_tmp();
        self.emitln(&format!("  {lp} = load i64, i64* {lpg}"));
        let rpg = self.fresh_tmp();
        self.emitln(&format!("  {rpg} = getelementptr {RTY}, {RTY}* {r_a}, i32 0, i32 1"));
        let rp = self.fresh_tmp();
        self.emitln(&format!("  {rp} = load i64, i64* {rpg}"));
        let peq = self.emit_payload_eq(ok_xiom, &lp, &rp, visited, depth);
        let pi = self.fresh_tmp();
        self.emitln(&format!("  {pi} = icmp ne i64 {peq}, 0"));
        self.emitln(&format!("  br i1 {pi}, label %{okok_b}, label %{okbad_b}"));
        // Err payload (field 2).
        self.emitln(&format!("\n{err_b}:"));
        let lpe = self.fresh_tmp();
        self.emitln(&format!("  {lpe} = getelementptr {RTY}, {RTY}* {l_a}, i32 0, i32 2"));
        let lep = self.fresh_tmp();
        self.emitln(&format!("  {lep} = load i64, i64* {lpe}"));
        let rpe = self.fresh_tmp();
        self.emitln(&format!("  {rpe} = getelementptr {RTY}, {RTY}* {r_a}, i32 0, i32 2"));
        let rep = self.fresh_tmp();
        self.emitln(&format!("  {rep} = load i64, i64* {rpe}"));
        let eeq = self.emit_payload_eq(err_xiom, &lep, &rep, visited, depth);
        let ei = self.fresh_tmp();
        self.emitln(&format!("  {ei} = icmp ne i64 {eeq}, 0"));
        self.emitln(&format!("  br i1 {ei}, label %{errok_b}, label %{errbad_b}"));
        self.emitln(&format!("\n{okok_b}:"));
        self.emitln(&format!("  br label %{merge_b}"));
        self.emitln(&format!("\n{okbad_b}:"));
        self.emitln(&format!("  br label %{merge_b}"));
        self.emitln(&format!("\n{errok_b}:"));
        self.emitln(&format!("  br label %{merge_b}"));
        self.emitln(&format!("\n{errbad_b}:"));
        self.emitln(&format!("  br label %{merge_b}"));
        self.emitln(&format!("\n{merge_b}:"));
        let phi = self.fresh_tmp();
        self.emitln(&format!(
            "  {phi} = phi i64 [ 0, %{ne_b} ], [ 1, %{okok_b} ], [ 0, %{okbad_b} ], [ 1, %{errok_b} ], [ 0, %{errbad_b} ]"
        ));
        phi
    }

    /// Compare two ERASED payload slots (i64). Boxed aggregates are
    /// unboxed and recursed; Str uses strcmp; floats are compared with
    /// fcmp after restoring their bits; other scalars compare bitwise.
    fn emit_payload_eq(
        &mut self,
        payload_xiom: &str,
        l_i64: &str,
        r_i64: &str,
        visited: &mut Vec<String>,
        depth: u32,
    ) -> String {
        if let Some(bllvm) = self
            .boxed_aggregate_llvm_for(payload_xiom)
            .or_else(|| self.registered_struct_llvm_for(payload_xiom))
        {
            if visited.iter().any(|t| t == payload_xiom) {
                // Recursive payload type: bounded handle identity.
                let (cmp, ext) = (self.fresh_tmp(), self.fresh_tmp());
                self.emitln(&format!("  {cmp} = icmp eq i64 {l_i64}, {r_i64}"));
                self.emitln(&format!("  {ext} = zext i1 {cmp} to i64"));
                return ext;
            }
            let lp = self.fresh_tmp();
            self.emitln(&format!("  {lp} = inttoptr i64 {l_i64} to {bllvm}*"));
            let lv = self.fresh_tmp();
            self.emitln(&format!("  {lv} = load {bllvm}, {bllvm}* {lp}"));
            let rp = self.fresh_tmp();
            self.emitln(&format!("  {rp} = inttoptr i64 {r_i64} to {bllvm}*"));
            let rv = self.fresh_tmp();
            self.emitln(&format!("  {rv} = load {bllvm}, {bllvm}* {rp}"));
            return self.emit_deep_eq(payload_xiom, &lv, &bllvm, &rv, &bllvm, visited, depth + 1);
        }
        let (base, _) = Self::deep_split_type(payload_xiom);
        match base.as_str() {
            "Str" => {
                let lp = self.fresh_tmp();
                self.emitln(&format!("  {lp} = inttoptr i64 {l_i64} to i8*"));
                let rp = self.fresh_tmp();
                self.emitln(&format!("  {rp} = inttoptr i64 {r_i64} to i8*"));
                self.emit_str_content_eq(&lp, "i8*", &rp, "i8*")
            }
            "Float64" => {
                let ld = self.fresh_tmp();
                self.emitln(&format!("  {ld} = bitcast i64 {l_i64} to double"));
                let rd = self.fresh_tmp();
                self.emitln(&format!("  {rd} = bitcast i64 {r_i64} to double"));
                self.emit_scalar_eq(&ld, "double", &rd, "double")
            }
            "Float32" => {
                let lt = self.fresh_tmp();
                self.emitln(&format!("  {lt} = trunc i64 {l_i64} to i32"));
                let lf = self.fresh_tmp();
                self.emitln(&format!("  {lf} = bitcast i32 {lt} to float"));
                let rt = self.fresh_tmp();
                self.emitln(&format!("  {rt} = trunc i64 {r_i64} to i32"));
                let rf = self.fresh_tmp();
                self.emitln(&format!("  {rf} = bitcast i32 {rt} to float"));
                self.emit_scalar_eq(&lf, "float", &rf, "float")
            }
            _ => {
                let (cmp, ext) = (self.fresh_tmp(), self.fresh_tmp());
                self.emitln(&format!("  {cmp} = icmp eq i64 {l_i64}, {r_i64}"));
                self.emitln(&format!("  {ext} = zext i1 {cmp} to i64"));
                ext
            }
        }
    }

    /// User struct / concrete container shell: derived `.eq` when
    /// registered, else field-by-field recursion over `type_meta`.
    fn emit_deep_aggregate_eq(
        &mut self,
        l_val: &str,
        l_llvm: &str,
        r_val: &str,
        r_llvm: &str,
        visited: &mut Vec<String>,
        depth: u32,
    ) -> String {
        if !l_llvm.starts_with("%struct.") || l_llvm.ends_with('*') || l_llvm != r_llvm {
            return self.emit_scalar_eq(l_val, l_llvm, r_val, r_llvm);
        }
        let sname = l_llvm.trim_start_matches("%struct.").to_string();
        let eq_fn = format!("{sname}.eq");
        if self.types.functions.contains_key(&eq_fn) {
            let out = self.fresh_tmp();
            self.emitln(&format!("  {out} = call i64 @{eq_fn}({l_llvm} {l_val}, {r_llvm} {r_val})"));
            return out;
        }
        let fields: Vec<(String, String)> = self
            .types
            .type_meta
            .get(&sname)
            .map(|m| m.fields.clone())
            .unwrap_or_default();
        if fields.is_empty() {
            return self.emit_identity_fallback(l_llvm, l_val, r_val);
        }
        let l_a = self.fresh_tmp();
        self.emitln(&format!("  {l_a} = alloca {l_llvm}"));
        self.emitln(&format!("  store {l_llvm} {l_val}, {l_llvm}* {l_a}"));
        let r_a = self.fresh_tmp();
        self.emitln(&format!("  {r_a} = alloca {r_llvm}"));
        self.emitln(&format!("  store {r_llvm} {r_val}, {r_llvm}* {r_a}"));
        let mut acc: Option<String> = None;
        for (i, (_fname, fxiom)) in fields.iter().enumerate() {
            let fllvm = self.field_llvm_type(&sname, i);
            let lg = self.fresh_tmp();
            self.emitln(&format!("  {lg} = getelementptr {l_llvm}, {l_llvm}* {l_a}, i32 0, i32 {i}"));
            let lv = self.fresh_tmp();
            self.emitln(&format!("  {lv} = load {fllvm}, {fllvm}* {lg}"));
            let rg = self.fresh_tmp();
            self.emitln(&format!("  {rg} = getelementptr {r_llvm}, {r_llvm}* {r_a}, i32 0, i32 {i}"));
            let rv = self.fresh_tmp();
            self.emitln(&format!("  {rv} = load {fllvm}, {fllvm}* {rg}"));
            let (fbase, _) = Self::deep_split_type(fxiom);
            let cmp = if matches!(fbase.as_str(), "Vec" | "Str" | "Option" | "Result")
                || (fllvm.starts_with("%struct.") && !fllvm.ends_with('*'))
            {
                self.emit_deep_eq(fxiom, &lv, &fllvm, &rv, &fllvm, visited, depth + 1)
            } else {
                self.emit_scalar_eq(&lv, &fllvm, &rv, &fllvm)
            };
            acc = Some(match acc {
                None => cmp,
                Some(prev) => {
                    let and = self.fresh_tmp();
                    self.emitln(&format!("  {and} = and i64 {prev}, {cmp}"));
                    and
                }
            });
        }
        match acc {
            Some(a) => a,
            None => {
                let one = self.fresh_tmp();
                self.emitln(&format!("  {one} = add i64 0, 1"));
                one
            }
        }
    }
}
