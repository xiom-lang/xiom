use super::IrEmitter;
use xiom_ast::*;

impl IrEmitter {
    /// When a value's LLVM type is a pointer to a Vec (e.g. `%struct.Vec*` from
    /// an `&mut Vec[T]` parameter), emit a load to get the actual Vec value.
    /// Returns `(value_name, "%struct.Vec")`. If the type is already a Vec value,
    /// returns the original value and type unchanged.
    /// 5c.29: Resolve a Vec receiver to a %struct.Vec* header POINTER for
    /// in-place mutation (insert/remove). For i64 container-field handles the
    /// box pointer itself is returned (authoritative — no store-back needed,
    /// second tuple element is false). Otherwise the header value is copied
    /// into a fresh alloca and the caller must store the mutated header back
    /// via `store_back_to_receiver` (second tuple element is true).
    pub(crate) fn resolve_vec_receiver_ptr(&mut self, receiver: &Expr) -> Result<(String, bool), String> {
        let (recv_val, recv_ty) = self.compile_expr(receiver)?;
        if recv_ty == "i64" && self.is_container_vec_field(receiver) {
            let boxp = self.fresh_tmp();
            self.emitln(&format!("  {boxp} = inttoptr i64 {recv_val} to %struct.Vec*"));
            return Ok((boxp, false));
        }
        // 5c.30: Indexed Vec element (e.g. outer[0] where outer: Vec[Vec[Int]]).
        // The index returns a loaded struct. For mutation (push), we need a pointer
        // into the outer Vec's data buffer so changes persist.
        if recv_ty == "i64" {
            if let Expr::Index(container, _, _) = receiver {
                let cont_ty = self.infer_llvm_type(container);
                if cont_ty == "%struct.Vec" || cont_ty.ends_with(".Vec") || cont_ty.contains("struct.Vec") {
                    let vp = self.fresh_tmp();
                    self.emitln(&format!("  {vp} = inttoptr i64 {recv_val} to %struct.Vec*"));
                    return Ok((vp, false));
                }
            }
        }
        // 5c.30: Struct-typed indexed element (compiled as %struct.Vec via memcpy).
        // We need a pointer to the element IN the data buffer, not the stack copy.
        if recv_ty == "%struct.Vec" {
            if let Expr::Index(container, idx, _) = receiver {
                let cont_ty = self.infer_llvm_type(container);
                if cont_ty == "%struct.Vec" || cont_ty.ends_with(".Vec") || cont_ty.contains("struct.Vec") {
                    // Re-resolve the element pointer for mutation
                    if let Some(elem_ptr) = self.resolve_index_elem_ptr(container, idx) {
                        let vp = self.fresh_tmp();
                        self.emitln(&format!("  {vp} = bitcast i8* {elem_ptr} to %struct.Vec*"));
                        return Ok((vp, false)); // mutations go directly to buffer
                    }
                }
            }
        }
        let (vec_val, _) = self.resolve_vec_value(&recv_val, &recv_ty);
        let slot = self.fresh_tmp();
        self.emitln(&format!("  {slot} = alloca %struct.Vec"));
        self.emit_vec_store_fields(&vec_val, &slot);
        Ok((slot, true))
    }

    /// 5c.30: Load a Vec element as an i64 Option payload. Scalars load
    /// directly (1/2/4/8-byte widths); STRUCT elements are heap-copied and
    /// the pointer stored as the payload — matching the val_to_i64 boxing
    /// convention consumed by FIELD-I64 access (`popped.unwrap().x`).
    pub(crate) fn emit_elem_payload_load(&mut self, container: &Expr, elem_ptr: &str, esz_val: &str) -> String {
        if self.resolve_vec_elem_type(container).is_some() {
            let raw = self.fresh_tmp();
            self.emitln(&format!("  {raw} = call i8* @malloc(i64 {esz_val})"));
            let ok = self.fresh_block("elem_box_ok");
            let fail = self.fresh_block("elem_box_trap");
            let chk = self.fresh_tmp();
            self.emitln(&format!("  {chk} = icmp eq i8* {raw}, null"));
            self.emitln(&format!("  br i1 {chk}, label %{fail}, label %{ok}"));
            self.emitln(&format!("\n{fail}:"));
            self.emitln("  call void @llvm.trap()");
            self.emitln("  unreachable");
            self.emitln(&format!("\n{ok}:"));
            self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {raw}, i8* {elem_ptr}, i64 {esz_val}, i1 false)"));
            let h = self.fresh_tmp();
            self.emitln(&format!("  {h} = ptrtoint i8* {raw} to i64"));
            return h;
        }
        self.emit_elem_load(elem_ptr, esz_val)
    }

    pub(crate) fn resolve_vec_value(&mut self, val: &str, ty: &str) -> (String, String) {
        if ty == "%struct.Vec" || ty.ends_with(".Vec") || ty.ends_with(".Slice") {
            return (val.to_string(), ty.to_string());
        }
        if (ty.starts_with("%struct.") && (ty.ends_with("Vec*") || ty.ends_with("Slice*")))
            || (ty.ends_with(".Vec*") || ty.ends_with(".Slice*"))
        {
            let inner_ty = ty.trim_end_matches('*');
            let loaded = self.emit_vec_load_fields(val);
            return (loaded, inner_ty.to_string());
        }
        (val.to_string(), ty.to_string())
    }

    /// 5c.29: Resolve a Vec receiver to a %struct.Vec SSA value, additionally
    /// dereferencing i64 container-field HANDLES (5c.28h): a generic container
    /// struct field (Vec[Int], ...) holds an i64 pointer to the heap-boxed
    /// Vec header, so an i64 receiver value must be inttoptr'd and loaded.
    pub(crate) fn resolve_vec_receiver(&mut self, receiver: &Expr, val: &str, ty: &str) -> (String, String) {
        let (v, t) = self.resolve_vec_value(val, ty);
        if t == "i64" && self.is_container_vec_field(receiver) {
            let vp = self.fresh_tmp();
            self.emitln(&format!("  {vp} = inttoptr i64 {v} to %struct.Vec*"));
            let vl = self.fresh_tmp();
            self.emitln(&format!("  {vl} = load volatile %struct.Vec, %struct.Vec* {vp}"));
            return (vl, "%struct.Vec".to_string());
        }
        // 5c.30: Indexed Vec element — inttoptr + load the struct.
        if t == "i64" {
            if let Expr::Index(container, _, _) = receiver {
                let cont_ty = self.infer_llvm_type(container);
                if cont_ty == "%struct.Vec" || cont_ty.ends_with(".Vec") || cont_ty.contains("struct.Vec") {
                    let vp = self.fresh_tmp();
                    self.emitln(&format!("  {vp} = inttoptr i64 {v} to %struct.Vec*"));
                    let vl = self.fresh_tmp();
                    self.emitln(&format!("  {vl} = load volatile %struct.Vec, %struct.Vec* {vp}"));
                    return (vl, "%struct.Vec".to_string());
                }
            }
        }
        (v, t)
    }

    /// Emit per-field stores of a `%struct.Vec` SSA value into an alloca.
    /// Uses extractvalue+GEP+store for each of the 4 fields to prevent LLVM
    /// from decomposing the aggregate store and skipping "dead" fields (5c.28).
    /// Emit per-field loads of a `%struct.Vec` from an alloca, returning the
    /// loaded SSA value. Prevents LLVM from decomposing the aggregate load
    /// into partial field reads that skip "dead" fields (5c.28).
    pub(crate) fn emit_vec_load_fields(&mut self, src_alloca: &str) -> String {
        let fd = self.fresh_tmp();
        let v0 = self.fresh_tmp();
        self.emitln(&format!("  {fd} = load i8*, i8** {src_alloca}"));
        self.emitln(&format!("  {v0} = insertvalue %struct.Vec undef, i8* {fd}, 0"));
        let mut prev = v0;
        for fi in 1..=3 {
            let fp = self.fresh_tmp();
            let gep = self.fresh_tmp();
            let vi = self.fresh_tmp();
            self.emitln(&format!("  {gep} = getelementptr %struct.Vec, %struct.Vec* {src_alloca}, i32 0, i32 {fi}"));
            self.emitln(&format!("  {fp} = load i64, i64* {gep}"));
            self.emitln(&format!("  {vi} = insertvalue %struct.Vec {prev}, i64 {fp}, {fi}"));
            prev = vi;
        }
        prev
    }

    /// Resolve the raw element pointer for `container[idx]` where container
    /// is a Vec. Returns an `i8*` pointing to the element data IN the buffer
    /// (no struct loading). Used by mutation operations (push/insert/remove)
    /// on indexed Vec elements so changes persist in the outer Vec.
    pub(crate) fn resolve_index_elem_ptr(&mut self, container: &Expr, idx: &Expr) -> Option<String> {
        // Compile the container to get its Vec struct
        let (cont_val, _) = self.compile_expr(container).ok()?;
        let slot = self.fresh_tmp();
        self.emitln(&format!("  {slot} = alloca %struct.Vec"));
        self.emit_vec_store_fields(&cont_val, slot.as_str());
        // Load elem_size
        let esz_gep = self.fresh_tmp();
        let esz = self.fresh_tmp();
        self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, %struct.Vec* {slot}, i32 0, i32 3"));
        self.emitln(&format!("  {esz} = load i64, i64* {esz_gep}"));
        // Load data ptr
        let data_gep = self.fresh_tmp();
        let data_ptr = self.fresh_tmp();
        self.emitln(&format!("  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {slot}, i32 0, i32 0"));
        self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
        // Compute element offset
        let (idx_val, _) = self.compile_expr(idx).ok()?;
        let byte_off = self.fresh_tmp();
        self.emitln(&format!("  {byte_off} = mul i64 {idx_val}, {esz}"));
        let elem_ptr = self.fresh_tmp();
        self.emitln(&format!("  {elem_ptr} = getelementptr i8, i8* {data_ptr}, i64 {byte_off}"));
        Some(elem_ptr)
    }

    /// Compile an array literal `[e1, e2, ...]` into a proper `%struct.Vec`
    /// value, handling malloc + per-element copy. Used when an array literal
    /// appears in a context that expects a Vec (e.g., `Some([1,2,3])`).
    pub(crate) fn compile_array_as_vec(&mut self, elems: &[xiom_ast::Expr], elem_xiom_type: &str) -> Result<(String, String), String> {
        let n = elems.len() as i64;
        // 5c.39: Resolve the element's LLVM type and byte size from the type
        // annotation, falling back to i64 (8 bytes) for unknown types.
        let elem_llvm_ty = self.llvm_type_for(elem_xiom_type).unwrap_or_else(|_| "i64".to_string());
        let elem_size: i64 = if elem_llvm_ty.starts_with("%struct.") {
            let struct_name = &elem_llvm_ty[8..]; // strip "%struct." prefix (8 chars)
            self.struct_byte_size(struct_name) as i64
        } else {
            // Scalar types: i64=8, double=8, float=4, i32=4, i16=2, i8=1
            match elem_llvm_ty.as_str() {
                "double" | "i64" => 8,
                "float" | "i32" => 4,
                "i16" => 2,
                "i8" | "i1" => 1,
                _ => 8,
            }
        };
        let initial_cap = n.max(16);
        let alloc_size = initial_cap * elem_size;
        // Allocate Vec struct on stack
        let vec_alloca = self.fresh_tmp();
        self.emitln(&format!("  {vec_alloca} = alloca %struct.Vec"));
        // malloc data buffer
        let data_ptr = self.fresh_tmp();
        self.emitln(&format!("  {data_ptr} = call i8* @malloc(i64 {alloc_size})"));
        let null_check = self.fresh_tmp();
        let ok_block = self.fresh_block("arr2vec_ok");
        let trap_block = self.fresh_block("arr2vec_trap");
        self.emitln(&format!("  {null_check} = icmp eq i8* {data_ptr}, null"));
        self.emitln(&format!("  br i1 {null_check}, label %{trap_block}, label %{ok_block}"));
        self.emitln(&format!("\n{trap_block}:"));
        self.emitln("  call void @llvm.trap()");
        self.emitln("  unreachable");
        self.emitln(&format!("\n{ok_block}:"));
        // Store data ptr, len, cap, elem_size
        let dg = self.fresh_tmp(); self.emitln(&format!("  {dg} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 0"));
        self.emitln(&format!("  store i8* {data_ptr}, i8** {dg}"));
        let lg = self.fresh_tmp(); self.emitln(&format!("  {lg} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 1"));
        self.emitln(&format!("  store i64 {n}, i64* {lg}"));
        let cg = self.fresh_tmp(); self.emitln(&format!("  {cg} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 2"));
        self.emitln(&format!("  store i64 {initial_cap}, i64* {cg}"));
        let eg = self.fresh_tmp(); self.emitln(&format!("  {eg} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 3"));
        self.emitln(&format!("  store i64 {elem_size}, i64* {eg}"));
        // Copy elements into buffer
        for (i, e) in elems.iter().enumerate() {
            let (ev, ety) = self.compile_expr(e)?;
            let offset = i as i64 * elem_size;
            let dest = self.fresh_tmp();
            self.emitln(&format!("  {dest} = getelementptr i8, i8* {data_ptr}, i64 {offset}"));
            // 5c.39: For struct elements, memcpy the full struct value.
            // For scalar elements, store as i64 via val_to_i64.
            if elem_llvm_ty.starts_with("%struct.") {
                let dest_typed = self.fresh_tmp();
                self.emitln(&format!("  {dest_typed} = bitcast i8* {dest} to {elem_llvm_ty}*"));
                let store_val = self.coerce_value(&ev, &ety, &elem_llvm_ty);
                self.emitln(&format!("  store {elem_llvm_ty} {store_val}, {elem_llvm_ty}* {dest_typed}"));
            } else {
                let ev_i64 = self.val_to_i64(&ev, &ety);
                let dest_i64 = self.fresh_tmp();
                self.emitln(&format!("  {dest_i64} = bitcast i8* {dest} to i64*"));
                self.emitln(&format!("  store i64 {ev_i64}, i64* {dest_i64}"));
            }
        }
        let loaded = self.emit_vec_load_fields(&vec_alloca);
        Ok((loaded, "%struct.Vec".to_string()))
    }

    /// Returns true if `container` is a field access on a struct and the
    /// field's type in type_meta is a generic container (Vec[..], Map[..], etc.)
    pub(crate) fn is_container_vec_field(&self, container: &Expr) -> bool {
        // 5c.30: locals bound to an i64 container handle (match-arm payload
        // bindings like `JsonValue.Array(ref mut items)`).
        if let Expr::Ident(id) = container {
            return self.local.local_vec_handle.contains_key(&id.name);
        }
        if let Expr::Field(base, field_expr, _) = container {
            if let Some(base_ty) = self.infer_struct_type_name(base) {
                for key in self.types.type_meta.keys() {
                    if key.ends_with(&base_ty) || key == &base_ty {
                        if let Some(meta) = self.types.type_meta.get(key) {
                            for (fname, ftype) in &meta.fields {
                                if fname == &field_expr.name {
                                    return ftype.contains('[');
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
        false
    }

    /// Returns the declared XIOM type name of field `field_idx` of `struct_name`
    /// from type_meta (e.g. "Vec[Int]"), using the same qualified-name fallbacks
    /// as `field_llvm_type`.
    /// 5c.30: Check whether a bare-name call inside a method should be
    /// resolved as an implicit-self method call (G-10). Called early in
    /// the Expr::Call handler.
    pub(crate) fn resolve_implicit_self_call(&self, fn_name: &str) -> Option<String> {
        let recv = self.fctx.current_receiver.as_ref()?;
        let key = format!("{recv}.{fn_name}");
        if self.types.functions.contains_key(&key) { return Some(key); }
        for k in self.types.functions.keys() {
            if k.ends_with(&format!(".{key}")) { return Some(k.clone()); }
        }
        None
    }

    /// 5c.29: Box a by-value container header (e.g. %struct.Vec) on the heap and
    /// return an i64 HANDLE (ptrtoint of the box). Generic container struct
    /// fields are declared as i64 handles (5c.28h); every reader dereferences
    /// the handle (Index handler inttoptr+load, val_to_struct memcpy), so
    /// writers must store a pointer to a stable heap header — storing the
    /// 32-byte header by value into the 8-byte slot corrupted the stack and
    /// made readers interpret element data as a Vec header (NET/VECTOR/HTTP/
    /// SQLITE ACCESS_VIOLATION).
    pub(crate) fn emit_box_struct_handle(&mut self, val: &str, struct_ty: &str) -> String {
        let type_name = struct_ty.trim_start_matches("%struct.").trim_end_matches('*');
        let size = self.struct_byte_size(type_name).max(32);
        let raw = self.fresh_tmp();
        self.emitln(&format!("  {raw} = call i8* @malloc(i64 {size})"));
        let ok = self.fresh_block("box_malloc_ok");
        let fail = self.fresh_block("box_malloc_trap");
        let chk = self.fresh_tmp();
        self.emitln(&format!("  {chk} = icmp eq i8* {raw}, null"));
        self.emitln(&format!("  br i1 {chk}, label %{fail}, label %{ok}"));
        self.emitln(&format!("\n{fail}:"));
        self.emitln("  call void @llvm.trap()");
        self.emitln("  unreachable");
        self.emitln(&format!("\n{ok}:"));
        let typed = self.fresh_tmp();
        self.emitln(&format!("  {typed} = bitcast i8* {raw} to {struct_ty}*"));
        self.emitln(&format!("  store {struct_ty} {val}, {struct_ty}* {typed}"));
        let handle = self.fresh_tmp();
        self.emitln(&format!("  {handle} = ptrtoint {struct_ty}* {typed} to i64"));
        handle
    }

    pub(crate) fn emit_vec_store_fields(&mut self, vec_val: &str, dest_alloca: &str) {
        for (fi, ty) in [(0, "i8*"), (1, "i64"), (2, "i64"), (3, "i64")] {
            let fval = self.fresh_tmp();
            let gep = self.fresh_tmp();
            self.emitln(&format!("  {fval} = extractvalue %struct.Vec {vec_val}, {fi}"));
            self.emitln(&format!("  {gep} = getelementptr %struct.Vec, %struct.Vec* {dest_alloca}, i32 0, i32 {fi}"));
            self.emitln(&format!("  store {ty} {fval}, {ty}* {gep}"));
        }
    }

    /// Emit a store of an i64 value at `dest` (i8*) using the element width from
    /// `esz_val` (loaded from Vec field 3). For elem_size == 8 (default Int/ptr),
    /// stores as i64. For elem_size < 8, truncates to the matching integer width
    /// to avoid overwriting adjacent elements.
    pub(crate) fn emit_elem_store(&mut self, val: &str, dest: &str, esz_val: &str) {
        // 5c.29: real 1/2/4/8-byte stores. The old code truncated EVERY
        // non-8 width to i8, destroying 4-byte elements (Float32 raw bits,
        // Int32/UInt32) and 2-byte elements (Int16/UInt16).
        let s1 = self.fresh_block("elem_store1");
        let s2 = self.fresh_block("elem_store2");
        let s4 = self.fresh_block("elem_store4");
        let s8 = self.fresh_block("elem_store8");
        let done = self.fresh_block("elem_store_done");
        self.emitln(&format!(
            "  switch i64 {esz_val}, label %{s8} [ i64 1, label %{s1}\n    i64 2, label %{s2}\n    i64 4, label %{s4} ]"
        ));
        // 1-byte path (UInt8/Int8/Char/Bool)
        self.emitln(&format!("\n{s1}:"));
        let t1 = self.fresh_tmp();
        self.emitln(&format!("  {t1} = trunc i64 {val} to i8"));
        self.emitln(&format!("  store i8 {t1}, i8* {dest}"));
        self.emitln(&format!("  br label %{done}"));
        // 2-byte path (Int16/UInt16)
        self.emitln(&format!("\n{s2}:"));
        let t2 = self.fresh_tmp();
        let p2 = self.fresh_tmp();
        self.emitln(&format!("  {t2} = trunc i64 {val} to i16"));
        self.emitln(&format!("  {p2} = bitcast i8* {dest} to i16*"));
        self.emitln(&format!("  store i16 {t2}, i16* {p2}"));
        self.emitln(&format!("  br label %{done}"));
        // 4-byte path (Int32/UInt32/Float32 raw bits)
        self.emitln(&format!("\n{s4}:"));
        let t4 = self.fresh_tmp();
        let p4 = self.fresh_tmp();
        self.emitln(&format!("  {t4} = trunc i64 {val} to i32"));
        self.emitln(&format!("  {p4} = bitcast i8* {dest} to i32*"));
        self.emitln(&format!("  store i32 {t4}, i32* {p4}"));
        self.emitln(&format!("  br label %{done}"));
        // 8-byte path (default: Int/ptr/double)
        self.emitln(&format!("\n{s8}:"));
        let dest64 = self.fresh_tmp();
        self.emitln(&format!("  {dest64} = bitcast i8* {dest} to i64*"));
        self.emitln(&format!("  store i64 {val}, i64* {dest64}"));
        self.emitln(&format!("  br label %{done}"));
        self.emitln(&format!("\n{done}:"));
    }

    /// Emit a load of an element from `src` (i8*) using the element width from
    /// `esz_val`. Returns the register holding the loaded-and-extended i64 value.
    /// Only called for primitive element types (Int, UInt8, etc.) -- struct
    /// elements use the direct struct load path in the Index handler.
    pub(crate) fn emit_elem_load(&mut self, src: &str, esz_val: &str) -> String {
        let result_slot = self.fresh_tmp();
        self.emitln(&format!("  {result_slot} = alloca i64"));  // in entry block
        // 5c.29: real 1/2/4/8-byte loads (the old code read EVERY non-8 width
        // as a single byte, destroying Float32/Int32/Int16 elements).
        let l1 = self.fresh_block("elem_load1");
        let l2 = self.fresh_block("elem_load2");
        let l4 = self.fresh_block("elem_load4");
        let l8 = self.fresh_block("elem_load8");
        let done = self.fresh_block("elem_load_done");
        self.emitln(&format!(
            "  switch i64 {esz_val}, label %{l8} [ i64 1, label %{l1}\n    i64 2, label %{l2}\n    i64 4, label %{l4} ]"
        ));
        // 1-byte path
        self.emitln(&format!("\n{l1}:"));
        let v1 = self.fresh_tmp();
        let e1 = self.fresh_tmp();
        self.emitln(&format!("  {v1} = load i8, i8* {src}"));
        self.emitln(&format!("  {e1} = zext i8 {v1} to i64"));
        self.emitln(&format!("  store i64 {e1}, i64* {result_slot}"));
        self.emitln(&format!("  br label %{done}"));
        // 2-byte path
        self.emitln(&format!("\n{l2}:"));
        let p2 = self.fresh_tmp();
        let v2 = self.fresh_tmp();
        let e2 = self.fresh_tmp();
        self.emitln(&format!("  {p2} = bitcast i8* {src} to i16*"));
        self.emitln(&format!("  {v2} = load i16, i16* {p2}"));
        self.emitln(&format!("  {e2} = zext i16 {v2} to i64"));
        self.emitln(&format!("  store i64 {e2}, i64* {result_slot}"));
        self.emitln(&format!("  br label %{done}"));
        // 4-byte path (Int32/UInt32/Float32 raw bits -- zext keeps the bit
        // pattern intact for the float bitcast done by the caller)
        self.emitln(&format!("\n{l4}:"));
        let p4 = self.fresh_tmp();
        let v4 = self.fresh_tmp();
        let e4 = self.fresh_tmp();
        self.emitln(&format!("  {p4} = bitcast i8* {src} to i32*"));
        self.emitln(&format!("  {v4} = load i32, i32* {p4}"));
        self.emitln(&format!("  {e4} = zext i32 {v4} to i64"));
        self.emitln(&format!("  store i64 {e4}, i64* {result_slot}"));
        self.emitln(&format!("  br label %{done}"));
        // 8-byte path (default)
        self.emitln(&format!("\n{l8}:"));
        let src64 = self.fresh_tmp();
        let load64 = self.fresh_tmp();
        self.emitln(&format!("  {src64} = bitcast i8* {src} to i64*"));
        self.emitln(&format!("  {load64} = load i64, i64* {src64}"));
        self.emitln(&format!("  store i64 {load64}, i64* {result_slot}"));
        self.emitln(&format!("  br label %{done}"));
        // Done
        self.emitln(&format!("\n{done}:"));
        let loaded = self.fresh_tmp();
        self.emitln(&format!("  {loaded} = load i64, i64* {result_slot}"));
        loaded
    }
}
