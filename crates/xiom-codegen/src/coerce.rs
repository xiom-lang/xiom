use super::IrEmitter;
use xiom_ast::*;
// B-001: llvm_consts not currently used here, but kept for future
// concrete type lowering that will need LLVM_STR_PTR etc.
#[allow(unused_imports)]
use crate::llvm_consts::*;

impl IrEmitter {
    pub(crate) fn coerce_arg_for_param(&mut self, arg_expr: &Expr, pre_val: &str, pre_ty: &str, param_ty: &str) -> String {
        if param_ty.ends_with('*') {
            let lvalue: Option<&Expr> = match arg_expr {
                Expr::Ref(i, _) | Expr::MutRef(i, _) => Some(i.as_ref()),
                Expr::Unary(UnaryOp::Ref, i, _) | Expr::Unary(UnaryOp::MutRef, i, _) => Some(i.as_ref()),
                _ => None,
            };
            if let Some(Expr::Ident(id)) = lvalue {
                if let Some((slot, slot_ty)) = self.lookup_local(&id.name).cloned() {
                    if slot_ty.ends_with('*') {
                        // Local already holds a pointer value: load and forward it.
                        if let Ok((v, t)) = self.compile_expr(arg_expr) {
                            return self.coerce_value(&v, &t, param_ty);
                        }
                    } else {
                        // Pass the address of the local's slot.
                        let addr_ty = format!("{slot_ty}*");
                        return self.coerce_value(&slot, &addr_ty, param_ty);
                    }
                }
            }
        }
        self.coerce_value(pre_val, pre_ty, param_ty)
    }

    pub(crate) fn coerce_value(&mut self, val: &str, from: &str, to: &str) -> String {
        if val.is_empty() {
            // A missing/void value can't be stored; substitute a typed default so
            // the sink (store/return/arg) stays well-formed. Void targets keep the
            // empty value (their sink omits the operand entirely).
            if to == "void" {
                return val.to_string();
            }
            return Self::default_const_for(to);
        }
        if from == to || to == "void" {
            return val.to_string();
        }
        let int_width = |t: &str| -> Option<u32> {
            match t {
                "i1" => Some(1),
                "i8" => Some(8),
                "i16" => Some(16),
                "i32" => Some(32),
                "i64" => Some(64),
                _ => None,
            }
        };
        // Integer <-> integer width conversions.
        if let (Some(a), Some(b)) = (int_width(from), int_width(to)) {
            let t = self.fresh_tmp();
            if b > a {
                let op = if from == "i1" || from == "i8" { "zext" } else { "sext" };
                self.emitln(&format!("  {t} = {op} {from} {val} to {to}"));
            } else {
                self.emitln(&format!("  {t} = trunc {from} {val} to {to}"));
            }
            return t;
        }
        // Integer <-> pointer.
        if to.ends_with('*') && from == "i64" {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = inttoptr i64 {val} to {to}"));
            return t;
        }
        if from.ends_with('*') && to == "i64" {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = ptrtoint {from} {val} to i64"));
            return t;
        }
        // i64 (heap pointer from val_to_i64) -> struct: inttoptr + load.
        // Handles Option/Result unwrap round-trip for struct payloads.
        if from == "i64" && to.starts_with('%') {
            let ptr = self.fresh_tmp();
            self.emitln(&format!("  {ptr} = inttoptr i64 {val} to {to}*"));
            let loaded = self.fresh_tmp();
            self.emitln(&format!("  {loaded} = load {to}, {to}* {ptr}"));
            return loaded;
        }
        // Pointer <-> pointer.
        if from.ends_with('*') && to.ends_with('*') {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = bitcast {from} {val} to {to}"));
            return t;
        }
        // Integer <-> double.
        if from == "i64" && to == "double" {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = sitofp i64 {val} to double"));
            return t;
        }
        if from == "double" && to == "i64" {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = fptosi double {val} to i64"));
            return t;
        }
        // Integer <-> float (Float32).
        if from == "i64" && to == "float" {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = sitofp i64 {val} to float"));
            return t;
        }
        if from == "float" && to == "i64" {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = fptosi float {val} to i64"));
            return t;
        }
        // float <-> double.
        if from == "float" && to == "double" {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = fpext float {val} to double"));
            return t;
        }
        if from == "double" && to == "float" {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = fptrunc double {val} to float"));
            return t;
        }
        // 5c-E: i8* buffer pointer -> %struct.Vec coercion.
        // Used when returning a Vec-typed array literal from a function
        // (e.g. `return out` where `out: Vec[Float32] = []` is i8*).
        if from == "i8*" && to == "%struct.Vec" {
            return self.val_to_struct(val, from, to);
        }
        // Typed struct value -> struct pointer: allocate a slot, store the value,
        // return the slot pointer.  e.g. `%struct.HttpHeaders -> %struct.HttpHeaders*`
        // when a method expects `&mut T` (pointer) but the caller has a T value.
        if to.ends_with('*') && from.starts_with("%struct.") {
            let base = to.trim_end_matches('*');
            if base.starts_with("%struct.") && (base == from || (base.len() > 8 && from.ends_with(&base[8..]))) {
                let slot = self.fresh_tmp();
                self.emitln(&format!("  {slot} = alloca {from}"));
                self.emitln(&format!("  store {from} {val}, {from}* {slot}"));
                return slot;
            }
        }
        // Typed struct pointer -> same struct value: load through the pointer.
        // e.g. `%struct.Agent* -> %struct.Agent` when calling agent_is_idle(a)
        // where the caller has `a: &mut Agent` (pointer) but the callee expects
        // `a: &Agent` (compiled as Agent value).
        if to.starts_with("%struct.") && from.ends_with('*') {
            let base = from.trim_end_matches('*'); // "%struct.Agent*" -> "%struct.Agent"
            if base.starts_with("%struct.") && (base == to || (base.len() > 8 && to.ends_with(&base[8..]))) {
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {to}, {from} {val}"));
                return loaded;
            }
        }
        // Non-struct scalar -> struct (e.g. i64 discriminant -> single-field enum).
        // If the scalar is i64, assume it's a pointer to a heap-allocated struct
        // (e.g. from Result::unwrap returning an enum value) and load it.
        if to.starts_with("%struct.") && !from.starts_with("%struct.") {
            if from == "i64" {
                let typed_ptr = self.fresh_tmp();
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {typed_ptr} = inttoptr i64 {val} to {to}*"));
                self.emitln(&format!("  {loaded} = load {to}, {to}* {typed_ptr}"));
                return loaded;
            }
            // Opaque pointer (ptr) or pointer types -> struct: load the struct value.
            // Also handle opaque ptr -> %struct.Vec (removed the i8* exclusion for ptr).
            if from == "ptr" {
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {to}, ptr {val}"));
                return loaded;
            }
            // pointer or pointer-like (T*) -> struct: load the value.
            // Exclude i8* -> Vec because that path requires val_to_struct's
            // array-buffer-to-Vec construction with proper field initialization.
            if from.ends_with('*') && !(from == "i8*" && (to == "%struct.Vec" || to.ends_with(".Vec"))) {
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {to}, {from} {val}"));
                return loaded;
            }
            return self.val_to_struct(val, from, to);
        }
        // Struct -> non-struct scalar: extract the leading i64 field (an enum
        // discriminant or an Option/Result's first slot), then coerce that i64 to
        // the target (e.g. Option -> i8 arg becomes field0 i64 -> i8). Handles
        // stdlib idioms where a single-scalar-backed struct is used as a scalar.
        // For Option/Result, extract field 1 (the value) not field 0 (discriminator).
        if from.starts_with("%struct.") && !to.starts_with("%struct.") {
            let type_name = &from[8..];
            let is_option_or_result = type_name == "Option" || type_name.ends_with(".Option")
                || type_name == "Result" || type_name.ends_with(".Result");
            let scalar = if is_option_or_result {
                self.extract_scalar_field1(val, from)
            } else {
                self.extract_scalar_field0(val, from)
            };
            return self.coerce_value(&scalar, "i64", to);
        }
        // No known cast — return unchanged (best effort).
        val.to_string()
    }

    /// Extract field 0 (the leading scalar — e.g. an enum discriminant or an
    /// Option/Result's first slot) from a by-value struct `val` of type
    /// `struct_ty`, returning the loaded `i64` scalar register. Used when a
    /// single-scalar-backed struct value appears in an integer context (e.g.
    /// `opt >= 0`). Returns `val` unchanged when `struct_ty` isn't a struct, and
    /// a `0` constant for empty (zero-field) structs which have no field 0.
    pub(crate) fn extract_scalar_field0(&mut self, val: &str, struct_ty: &str) -> String {
        if !struct_ty.starts_with("%struct.") {
            return val.to_string();
        }
        // Empty (zero-sized) structs have no field 0 — GEP would be invalid.
        let type_name = &struct_ty[8..];
        let is_empty = self.types.type_meta.get(type_name).map(|m| m.fields.is_empty()).unwrap_or(false);
        if is_empty {
            return "0".to_string();
        }
        let slot = self.fresh_tmp();
        self.emitln(&format!("  {slot} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} {val}, {struct_ty}* {slot}"));
        let gep = self.fresh_tmp();
        self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {slot}, i32 0, i32 0"));
        let loaded = self.fresh_tmp();
        self.emitln(&format!("  {loaded} = load i64, i64* {gep}"));
        loaded
    }

    /// Same as extract_scalar_field0 but extracts field 1 (used for Option/Result
    /// value comparisons like `char_at(s,i) == '.'`).
    pub(crate) fn extract_scalar_field1(&mut self, val: &str, struct_ty: &str) -> String {
        if !struct_ty.starts_with("%struct.") {
            return val.to_string();
        }
        let type_name = &struct_ty[8..];
        let is_empty = self.types.type_meta.get(type_name).map(|m| m.fields.is_empty()).unwrap_or(false);
        if is_empty {
            return "0".to_string();
        }
        let slot = self.fresh_tmp();
        self.emitln(&format!("  {slot} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} {val}, {struct_ty}* {slot}"));
        let gep = self.fresh_tmp();
        self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {slot}, i32 0, i32 1"));
        let loaded = self.fresh_tmp();
        self.emitln(&format!("  {loaded} = load i64, i64* {gep}"));
        loaded
    }

    /// Convert a value to i64 via bitcast or ptrtoint if needed
    pub(crate) fn val_to_i64(&mut self, val: &str, ty: &str) -> String {
        if ty == "void" {
            return "0".to_string();
        }
        if ty == "double" {
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {bc} = bitcast double {val} to i64"));
            bc
        } else if ty == "float" {
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {bc} = bitcast float {val} to i32"));
            let ext = self.fresh_tmp();
            self.emitln(&format!("  {ext} = zext i32 {bc} to i64"));
            ext
        } else if ty == "i8*" || ty.contains('*') {
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {bc} = ptrtoint {ty} {val} to i64"));
            bc
        } else if ty == "i1" || ty == "i8" {
            // Narrow unsigned integer (Bool/Char/UInt8) -> i64: zero-extend.
            let ext = self.fresh_tmp();
            self.emitln(&format!("  {ext} = zext {ty} {val} to i64"));
            ext
        } else if ty == "i16" || ty == "i32" {
            // Narrow signed integer (Int16/Int32) -> i64: sign-extend.
            let ext = self.fresh_tmp();
            self.emitln(&format!("  {ext} = sext {ty} {val} to i64"));
            ext
        } else if ty.starts_with('%') {
            // Compute the actual size of the struct type using GEP trick:
            // `getelementptr %T, %T* null, i32 1` gives the byte offset
            // of element 1, which equals sizeof(T). This is correct even
            // when fields are larger than i64 (e.g. Vec = 24, Map = 48).
            let size_i64 = self.fresh_tmp();
            let null_ptr = self.fresh_tmp();
            self.emitln(&format!("  {null_ptr} = getelementptr {ty}, {ty}* null, i32 1"));
            self.emitln(&format!("  {size_i64} = ptrtoint {ty}* {null_ptr} to i64"));
            let malloc_ptr = self.fresh_tmp();
            let typed_ptr = self.fresh_tmp();
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {malloc_ptr} = call i8* @malloc(i64 {size_i64})"));
            self.emitln(&format!("  {typed_ptr} = bitcast i8* {malloc_ptr} to {ty}*"));
            self.emitln(&format!("  store {ty} {val}, {ty}* {typed_ptr}"));
            self.emitln(&format!("  {bc} = ptrtoint {ty}* {typed_ptr} to i64"));
            bc
        } else {
            val.to_string()
        }
    }

    /// Convert a value to i8* via bitcast (for pointers) or inttoptr (for ints)
    pub(crate) fn val_to_i8ptr(&mut self, val: &str, ty: &str) -> String {
        if ty == "i8*" {
            val.to_string()
        } else if ty.contains('*') {
            let cast = self.fresh_tmp();
            self.emitln(&format!("  {cast} = bitcast {ty} {val} to i8*"));
            cast
        } else {
            let i64_val = self.val_to_i64(val, ty);
            let cast = self.fresh_tmp();
            self.emitln(&format!("  {cast} = inttoptr i64 {i64_val} to i8*"));
            cast
        }
    }

    pub(crate) fn val_to_struct(&mut self, val: &str, val_ty: &str, struct_ty: &str) -> String {
        let alloca = self.fresh_tmp();
        self.emitln(&format!("  {alloca} = alloca {struct_ty}"));

        // i8* array-buffer -> %struct.Vec: only when val originates from an
        // Expr::Array (tracked in array_value_regs). The buffer has layout
        // [length:i64, elem0, elem1, ...]. Construct a proper Vec with a
        // heap copy so the Vec can be safely modified/passed.
        let type_name = &struct_ty[8..];
        let is_vec = type_name == "Vec" || type_name.ends_with(".Vec");
        if is_vec && val_ty == "i8*" && self.local.array_value_regs.contains(val) {
            // (existing array-buffer-to-Vec code, unchanged)
            let len_slot = self.fresh_tmp();
            self.emitln(&format!("  {len_slot} = bitcast i8* {val} to i64*"));
            let len_val = self.fresh_tmp();
            self.emitln(&format!("  {len_val} = load i64, i64* {len_slot}"));
            let byte_count = self.fresh_tmp();
            self.emitln(&format!("  {byte_count} = mul i64 {len_val}, 8"));
            let heap_copy = self.fresh_tmp();
            self.emitln(&format!("  {heap_copy} = call i8* @malloc(i64 {byte_count})"));
            let ok = self.fresh_block("arr_to_vec_ok");
            let fail = self.fresh_block("arr_to_vec_fail");
            let chk = self.fresh_tmp();
            self.emitln(&format!("  {chk} = icmp eq i8* {heap_copy}, null"));
            self.emitln(&format!("  br i1 {chk}, label %{fail}, label %{ok}"));
            self.emitln(&format!("\n{fail}:"));
            self.emitln("  call void @llvm.trap()");
            self.emitln("  unreachable");
            self.emitln(&format!("\n{ok}:"));
            let src = self.fresh_tmp();
            self.emitln(&format!("  {src} = getelementptr i8, i8* {val}, i64 8"));
            self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {heap_copy}, i8* {src}, i64 {byte_count}, i1 false)"));
            let g0 = self.fresh_tmp();
            self.emitln(&format!("  {g0} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
            self.emitln(&format!("  store i8* {heap_copy}, i8** {g0}"));
            let g1 = self.fresh_tmp();
            self.emitln(&format!("  {g1} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 1"));
            self.emitln(&format!("  store i64 {len_val}, i64* {g1}"));
            let g2 = self.fresh_tmp();
            self.emitln(&format!("  {g2} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 2"));
            self.emitln(&format!("  store i64 {len_val}, i64* {g2}"));
            let g3 = self.fresh_tmp();
            self.emitln(&format!("  {g3} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 3"));
            self.emitln(&format!("  store i64 8, i64* {g3}"));
            let loaded = self.fresh_tmp();
            self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {alloca}"));
            return loaded;
        }
        // 5c-E: i8* -> %struct.Vec from non-array sources (e.g. Vec param
        // passed to extern function expecting %struct.Vec by value).
        // Construct a minimal Vec struct from the buffer pointer.
        if is_vec && (val_ty == "i8*" || val_ty.ends_with('*')) {
            let g0 = self.fresh_tmp();
            self.emitln(&format!("  {g0} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
            self.emitln(&format!("  store i8* {val}, i8** {g0}"));
            let g1 = self.fresh_tmp();
            self.emitln(&format!("  {g1} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 1"));
            self.emitln(&format!("  store i64 0, i64* {g1}"));
            let g2 = self.fresh_tmp();
            self.emitln(&format!("  {g2} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 2"));
            self.emitln(&format!("  store i64 0, i64* {g2}"));
            let g3 = self.fresh_tmp();
            self.emitln(&format!("  {g3} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 3"));
            self.emitln(&format!("  store i64 0, i64* {g3}"));
            let loaded = self.fresh_tmp();
            self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {alloca}"));
            return loaded;
        }

        if val_ty.starts_with('%') {
            let ptr = self.fresh_tmp();
            self.emitln(&format!("  {ptr} = bitcast {struct_ty}* {alloca} to i64*"));
            let i64_val = self.val_to_i64(val, val_ty);
            self.emitln(&format!("  store i64 {i64_val}, i64* {ptr}"));
        } else if val_ty == "i64" {
            // For multi-field structs loaded from Vec (heap pointer from val_to_i64),
            // memcpy the full struct from the heap instead of storing a single i64.
            let num_fields = self.types.types.get(type_name)
                .or_else(|| {
                    let suffix = format!(".{}", type_name);
                    self.types.types.keys().find(|k| k.ends_with(&suffix))
                        .and_then(|k| self.types.types.get(k))
                })
                .map(|f| f.len())
                .unwrap_or(1);
            if num_fields > 1 {
                let src = self.fresh_tmp();
                self.emitln(&format!("  {src} = inttoptr i64 {val} to i8*"));
                let dst = self.fresh_tmp();
                self.emitln(&format!("  {dst} = bitcast {struct_ty}* {alloca} to i8*"));
                // 5c.30: real layout size (nested by-value structs count fully).
                let sz = self.struct_byte_size(type_name);
                self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {dst}, i8* {src}, i64 {sz}, i1 false)"));
            } else {
                let ptr = self.fresh_tmp();
                self.emitln(&format!("  {ptr} = bitcast {struct_ty}* {alloca} to i64*"));
                self.emitln(&format!("  store {val_ty} {val}, {val_ty}* {ptr}"));
            }
        } else {
            let ptr = self.fresh_tmp();
            self.emitln(&format!("  {ptr} = bitcast {struct_ty}* {alloca} to {val_ty}*"));
            self.emitln(&format!("  store {val_ty} {val}, {val_ty}* {ptr}"));
        }
        let loaded = self.fresh_tmp();
        self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {alloca}"));
        loaded
    }
}
