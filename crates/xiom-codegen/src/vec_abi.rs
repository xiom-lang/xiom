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
    ///
    /// For simple local-variable Ident receivers, this creates a new alloca
    /// each call — which leaks stack space when called in loops (R4 fix).
    /// Prefer `resolve_vec_push_ptr` for push/in-place mutating operations
    /// that may execute inside loops.
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
        // BUG 34: `bs[i].push(x)` — an INDEXED element of a Vec whose elements
        // are Vecs. The element must be mutated IN the outer data buffer: the
        // element-address path (GEP) is authoritative for ALL Index receivers
        // on Vec containers, whether the compiled element is an i64 or a
        // %struct.Vec — the old i64 branch inttoptr'd the ELEMENT VALUE as a
        // pointer (mutations hit garbage) and the %struct.Vec branch needed
        // the (often mis-inferred) struct type to fire at all.
        if let Expr::Index(container, idx, _) = receiver {
            let cont_ty = self.infer_llvm_type(container);
            if Self::is_llvm_struct_named(&cont_ty, "Vec") {
                if let Some(elem_ptr) = self.resolve_index_elem_ptr(container, idx) {
                    let vp = self.fresh_tmp();
                    self.emitln(&format!("  {vp} = bitcast i8* {elem_ptr} to %struct.Vec*"));
                    return Ok((vp, false)); // mutations go directly to buffer
                }
            }
        }
        if recv_ty == "i64" {
            if let Expr::Index(container, _, _) = receiver {
                let cont_ty = self.infer_llvm_type(container);
                if Self::is_llvm_struct_named(&cont_ty, "Vec") {
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
                if Self::is_llvm_struct_named(&cont_ty, "Vec") {
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

    /// R4 fix: Resolve a Vec receiver to a %struct.Vec* pointer for push/in-place
    /// mutation WITHOUT creating a per-call scratch alloca. For simple local
    /// variables (Ident), returns the local's original alloca directly — avoiding
    /// the dynamic-alloca-in-loop stack bloat that caused ACCESS_VIOLATION for
    /// large Vec push loops (>100K iterations).
    ///
    /// Returns (pointer_to_vec, needs_store_back). When needs_store_back is false,
    /// the caller is already working on the authoritative storage and no
    /// store_back_to_receiver call is needed.
    pub(crate) fn resolve_vec_push_ptr(&mut self, receiver: &Expr) -> Result<(String, bool), String> {
        // For simple local variables, use the receiver's original alloca directly.
        // This is the critical fix: avoids creating a new 32-byte alloca every
        // push iteration, which accumulates unbounded stack usage in loops.
        if let Expr::Ident(id) = receiver {
            if let Some((alloca, llvm_ty)) = self.lookup_local(&id.name).cloned() {
                // If the local is a direct %struct.Vec alloca, use it as-is.
                if llvm_ty == "%struct.Vec" {
                    // The local's alloca already stores a %struct.Vec value.
                    // Push modifies it in-place — no copy needed, no store-back needed.
                    return Ok((alloca, false));
                }
                // If the local is a pointer to Vec (&mut Vec[T]), we need to
                // load the pointer and work through it. Delegate to the full path.
            }
        }
        // For complex receivers (container fields, indexed elements, struct
        // fields), fall back to the existing resolve_vec_receiver_ptr which
        // correctly handles heap-boxed and buffer-inlined Vecs.
        self.resolve_vec_receiver_ptr(receiver)
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
                if Self::is_llvm_struct_named(&cont_ty, "Vec") {
                    let vp = self.fresh_tmp();
                    self.emitln(&format!("  {vp} = inttoptr i64 {v} to %struct.Vec*"));
                    let vl = self.fresh_tmp();
                    self.emitln(&format!("  {vl} = load volatile %struct.Vec, %struct.Vec* {vp}"));
                    return (vl, "%struct.Vec".to_string());
                }
            }
        }
        // M33: Handle i64 returned from Result.unwrap() — inttoptr+load
        // the boxed Vec struct so the caller can use it as a Vec.
        if t == "i64" && self.receiver_is_unwrap_of_vec(receiver) {
            let vp = self.fresh_tmp();
            self.emitln(&format!("  {vp} = inttoptr i64 {v} to %struct.Vec*"));
            let vl = self.fresh_tmp();
            self.emitln(&format!("  {vl} = load volatile %struct.Vec, %struct.Vec* {vp}"));
            return (vl, "%struct.Vec".to_string());
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

    /// True when `ty` is the LLVM type of a builtin container VALUE or POINTER
    /// whose leaf struct name is exactly `name` — bare ("%struct.Vec"),
    /// module-qualified ("%struct.xiom.collections.Vec"), container-args
    /// ("%struct.Vec[Int]"/"%struct.Slice[Float64]"), or pointer forms.
    /// round-8 (geom regression): the args-embedded form MUST match — an
    /// indexed Vec element (`ac[0].len()`) types as "%struct.Vec[Int]" and
    /// the OLD leaf test ("Vec[Int]" ≠ "Vec") skipped the inline Vec.len
    /// handler, falling into the generic leaf-match which mono'd a garbage
    /// "@Vec[Int].len_Int" symbol (brackets are invalid in LLVM identifiers).
    /// The round-7 `ty.contains("struct.Vec")` test matched VecDeque etc.
    /// (the bug this helper replaced) — splitting on '[' keeps both fixed.
    pub(crate) fn is_llvm_struct_named(ty: &str, name: &str) -> bool {
        let base = ty.trim_end_matches('*');
        let leaf = base.strip_prefix("%struct.").unwrap_or(base);
        let leaf = leaf.rsplit('.').next().unwrap_or(leaf);
        let leaf = leaf.split('[').next().unwrap_or(leaf);
        leaf == name
    }

    /// round-8 (rw1): AUTO-DEREF a reference-typed operand in VALUE positions.
    /// `&T` compiles to the T slot's ADDRESS (i64 bits) — whether from
    /// `Some(&items[i])` payloads, `&local`, or `&self.field`. Consumers that
    /// treat the value as a T (strcmp content compare, icmp, arithmetic) must
    /// LOAD THROUGH the address once; the old code inttoptr'd the address and
    /// read garbage (rand.weighted_pick's Option<&Str> payload compared via
    /// strcmp on the SLOT ADDRESS bytes). Returns (deref_val, pointee_ty) when
    /// `expr`'s registered XIOM type is a reference; otherwise unchanged.
    pub(crate) fn auto_deref_ref(&mut self, expr: &Expr, val: &str, llvm_ty: &str) -> (String, String) {
        let (inner_name, is_ref) = match expr {
            Expr::Ident(id) => {
                let xiom = self.local.local_xiom_types.get(&id.name).cloned();
                if xiom.as_deref().map_or(false, |t| t.starts_with('&')) {
                    // &T-typed: params (decl.rs keeps the &), payload bindings
                    // (stmt.rs scrutinee_payload fallback), annotated locals.
                    (xiom.unwrap()[1..].trim_start().to_string(), true)
                } else if self.local.ref_locals.contains(&id.name) {
                    // BUG 44 ref-locals (`var r = &s`, `var r: &Str = ...`):
                    // the tracked name is the POINTEE (type_from_ast strips &).
                    (xiom.unwrap_or_else(|| "Int".to_string()), true)
                } else {
                    (String::new(), false)
                }
            }
            Expr::Paren(inner, _) => match inner.as_ref() {
                Expr::Ident(id) => {
                    let xiom = self.local.local_xiom_types.get(&id.name).cloned();
                    if xiom.as_deref().map_or(false, |t| t.starts_with('&')) {
                        (xiom.unwrap()[1..].trim_start().to_string(), true)
                    } else if self.local.ref_locals.contains(&id.name) {
                        (xiom.unwrap_or_else(|| "Int".to_string()), true)
                    } else {
                        (String::new(), false)
                    }
                }
                _ => (String::new(), false),
            },
            _ => (String::new(), false),
        };
        if !is_ref {
            return (val.to_string(), llvm_ty.to_string());
        }
        let inner = inner_name.strip_prefix("mut ").unwrap_or(&inner_name);
        let pointee = match self.llvm_type_for(inner) {
            Ok(t) => t,
            Err(_) => return (val.to_string(), llvm_ty.to_string()),
        };
        // The address may already be pointer-typed (i8** param, i64* param,
        // i8* ref-local) or raw i64 bits (payload slots) — pointer-to-pointer
        // casts must be bitcast, not inttoptr (invalid "cast from ptr to ptr").
        let p = self.fresh_tmp();
        if llvm_ty.ends_with('*') {
            self.emitln(&format!("  {p} = bitcast {llvm_ty} {val} to {pointee}*"));
        } else {
            self.emitln(&format!("  {p} = inttoptr {llvm_ty} {val} to {pointee}*"));
        }
        let v = self.fresh_tmp();
        self.emitln(&format!("  {v} = load {pointee}, {pointee}* {p}"));
        (v, pointee)
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
            // BUG 29 (BUG 27 #12): tuple field of a BOXED struct local
            // (`pair.1` where `pair` came from `Result[(Vec,Vec),Str].
            // unwrap()`). infer_llvm_type returns i64 for the boxed handle,
            // so the Vec.len() handler must classify via local_boxed_struct:
            // resolve the tuple's field type and check for a container.
            if let Expr::Ident(base_id) = base.as_ref() {
                if let Some(tn) = self.local.local_boxed_struct.get(&base_id.name).cloned() {
                    let field_names = self.types.types.get(&tn)
                        .or_else(|| {
                            let suffix = format!(".{tn}");
                            self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix))
                                .and_then(|k| self.types.types.get(&k))
                        });
                    if let Some(names) = field_names {
                        if let Some(fi) = IrEmitter::resolve_field_index(&names, &field_expr.name) {
                            if let Some(meta) = self.types.type_meta.get(&tn) {
                                if let Some((_, ftype)) = meta.fields.get(fi) {
                                    // Container fields may be "Vec[UInt8]" OR the
                                    // erased bare "Vec" (tuple element typing uses
                                    // xiom_type_name_from_llvm which strips args).
                                    let base = ftype.split('[').next().unwrap_or(ftype);
                                    return base == "Vec" || base == "Slice" || base == "Array" || base == "Map" || base == "Set";
                                }
                            }
                        }
                    }
                }
            }
            if let Some(base_ty) = self.infer_struct_type_name(base) {
                // round-7 (ve2 regression): two structs can share a leaf name
                // (collect.Graph vs math.graph_theory.Graph) — the OLD loop
                // `break`t at the FIRST suffix-matching key, so a field of the
                // OTHER type ("edges" of graph_theory.Graph) was never found
                // and `g.edges.push(...)` missed the inline Vec handler (fell
                // into the generic leaf-match → @Graph.push_Int mono with a
                // literal 0 receiver → invalid IR). Search ALL matching keys.
                for key in self.types.type_meta.keys() {
                    if key.ends_with(&base_ty) || key == base_ty {
                        if let Some(meta) = self.types.type_meta.get(&key) {
                            for (fname, ftype) in &meta.fields {
                                if fname == &field_expr.name {
                                    // round-9 (Set ABI): the field must be an
                                    // INLINE-handled container (Vec/Slice/Array).
                                    // The OLD `ftype.contains('[')` matched ANY
                                    // generic field — a `Set[Int]` field made
                                    // `holder.s.insert(...)` fire the inline
                                    // Vec.insert on the %struct.Set (invalid IR).
                                    // Map/Set fields route through their own
                                    // generic-mono methods.
                                    let base = ftype.split('[').next().unwrap_or(ftype);
                                    return base == "Vec" || base == "Slice" || base == "Array";
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// M33: Check if a method-call receiver is the result of `.unwrap()`
    /// on a Result/Option containing a Vec-type payload.
    pub(crate) fn receiver_is_unwrap_of_vec(&self, receiver: &Expr) -> bool {
        if let Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) = receiver {
            if let Expr::Field(base, field, _) = func.as_ref() {
                if field.name == "unwrap" || field.name == "unwrap_or" {
                    if let Expr::Ident(id) = base.as_ref() {
                        if let Some(t) = self.local.local_opt_payload.get(&id.name) {
                            return t.starts_with("Vec[") || t.contains(".Vec");
                        }
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
        // 5c.29: real 1/2/4/8-byte stores. Uses if/else chain instead of
        // switch i64 to avoid LLVM -O0 codegen bugs (ACCESS_VIOLATION with
        // large Vecs on Windows).
        let s1 = self.fresh_block("elem_store1");
        let s2 = self.fresh_block("elem_store2");
        let s4 = self.fresh_block("elem_store4");
        let s8 = self.fresh_block("elem_store8");
        let done = self.fresh_block("elem_store_done");

        // if/else chain instead of switch i64 (avoids -O0 codegen bugs)
        let c1 = self.fresh_tmp();
        let c2 = self.fresh_tmp();
        let c4 = self.fresh_tmp();
        self.emitln(&format!("  {c1} = icmp eq i64 {esz_val}, 1"));
        self.emitln(&format!("  br i1 {c1}, label %{s1}, label %{s2}_chk"));
        self.emitln(&format!("\n{s2}_chk:"));
        self.emitln(&format!("  {c2} = icmp eq i64 {esz_val}, 2"));
        self.emitln(&format!("  br i1 {c2}, label %{s2}, label %{s4}_chk"));
        self.emitln(&format!("\n{s4}_chk:"));
        self.emitln(&format!("  {c4} = icmp eq i64 {esz_val}, 4"));
        self.emitln(&format!("  br i1 {c4}, label %{s4}, label %{s8}"));

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
    /// OPT-R6: Uses a phi node (not alloca+store+load) to merge the switch arms.
    /// This saves 3 LLVM instructions per element load (alloca + store + reload).
    /// In packet-processing loops with 46M element accesses, this eliminates
    /// ~138M redundant instructions.
    pub(crate) fn emit_elem_load(&mut self, src: &str, esz_val: &str) -> String {
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
        self.emitln(&format!("  br label %{done}"));
        // 2-byte path
        self.emitln(&format!("\n{l2}:"));
        let p2 = self.fresh_tmp();
        let v2 = self.fresh_tmp();
        let e2 = self.fresh_tmp();
        self.emitln(&format!("  {p2} = bitcast i8* {src} to i16*"));
        self.emitln(&format!("  {v2} = load i16, i16* {p2}"));
        self.emitln(&format!("  {e2} = zext i16 {v2} to i64"));
        self.emitln(&format!("  br label %{done}"));
        // 4-byte path
        self.emitln(&format!("\n{l4}:"));
        let p4 = self.fresh_tmp();
        let v4 = self.fresh_tmp();
        let e4 = self.fresh_tmp();
        self.emitln(&format!("  {p4} = bitcast i8* {src} to i32*"));
        self.emitln(&format!("  {v4} = load i32, i32* {p4}"));
        self.emitln(&format!("  {e4} = zext i32 {v4} to i64"));
        self.emitln(&format!("  br label %{done}"));
        // 8-byte path (default)
        self.emitln(&format!("\n{l8}:"));
        let src64 = self.fresh_tmp();
        let load64 = self.fresh_tmp();
        self.emitln(&format!("  {src64} = bitcast i8* {src} to i64*"));
        self.emitln(&format!("  {load64} = load i64, i64* {src64}"));
        self.emitln(&format!("  br label %{done}"));
        // Done — phi node merges the four paths (no alloca+store+load)
        self.emitln(&format!("\n{done}:"));
        let loaded = self.fresh_tmp();
        self.emitln(&format!("  {loaded} = phi i64 [ {e1}, %{l1} ], [ {e2}, %{l2} ], [ {e4}, %{l4} ], [ {load64}, %{l8} ]"));
        loaded
    }

    /// 5c.39: In-place insertion sort for Vec[Int]. Sorts the Vec's data buffer
    /// by comparing i64 element values. Stores back to receiver.
    pub(crate) fn emit_vec_sort(&mut self, receiver: &Expr, _recv_ty: String) -> Result<(), String> {
        let (recv_val, recv_actual_ty) = self.compile_expr(receiver)?;
        let (recv_vec, _) = self.resolve_vec_receiver(receiver, &recv_val, &recv_actual_ty);
        let vec_alloca = self.fresh_tmp();
        self.emitln(&format!("  {vec_alloca} = alloca %struct.Vec"));
        self.emit_vec_store_fields(&recv_vec, &vec_alloca);
        let dg = self.fresh_tmp(); let dp = self.fresh_tmp();
        self.emitln(&format!("  {dg} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 0"));
        self.emitln(&format!("  {dp} = load i8*, i8** {dg}"));
        let lg = self.fresh_tmp(); let lv = self.fresh_tmp();
        self.emitln(&format!("  {lg} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 1"));
        self.emitln(&format!("  {lv} = load i64, i64* {lg}"));
        let eszg = self.fresh_tmp(); let eszv = self.fresh_tmp();
        self.emitln(&format!("  {eszg} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 3"));
        self.emitln(&format!("  {eszv} = load i64, i64* {eszg}"));
        // Skip sort if len <= 1
        let skip = self.fresh_block("sort_skip");
        let sort = self.fresh_block("sort_start");
        let c0 = self.fresh_tmp();
        self.emitln(&format!("  {c0} = icmp sle i64 {lv}, 1"));
        self.emitln(&format!("  br i1 {c0}, label %{skip}, label %{sort}"));
        self.emitln(&format!("\n{sort}:"));
        // Insertion sort
        let i_slot = self.fresh_tmp();
        self.emitln(&format!("  {i_slot} = alloca i64"));
        self.emitln(&format!("  store i64 1, i64* {i_slot}"));
        let oh = self.fresh_block("sort_outer");
        let ob = self.fresh_block("sort_ob");
        let od = self.fresh_block("sort_od");
        self.emitln(&format!("  br label %{oh}"));
        self.emitln(&format!("\n{oh}:"));
        let iv = self.fresh_tmp();
        self.emitln(&format!("  {iv} = load i64, i64* {i_slot}"));
        let ic = self.fresh_tmp();
        self.emitln(&format!("  {ic} = icmp slt i64 {iv}, {lv}"));
        self.emitln(&format!("  br i1 {ic}, label %{ob}, label %{od}"));
        self.emitln(&format!("\n{ob}:"));
        // key = data[i]
        let ko = self.fresh_tmp();
        self.emitln(&format!("  {ko} = mul i64 {iv}, {eszv}"));
        let kp = self.fresh_tmp(); self.emitln(&format!("  {kp} = getelementptr i8, i8* {dp}, i64 {ko}"));
        let kpi = self.fresh_tmp(); self.emitln(&format!("  {kpi} = bitcast i8* {kp} to i64*"));
        let kv = self.fresh_tmp(); self.emitln(&format!("  {kv} = load i64, i64* {kpi}"));
        // j = i - 1
        let js = self.fresh_tmp(); self.emitln(&format!("  {js} = alloca i64"));
        let ji = self.fresh_tmp();
        self.emitln(&format!("  {ji} = sub i64 {iv}, 1"));
        self.emitln(&format!("  store i64 {ji}, i64* {js}"));
        let ih = self.fresh_block("sort_inner");
        let ib = self.fresh_block("sort_ib");
        let id = self.fresh_block("sort_id");
        self.emitln(&format!("  br label %{ih}"));
        self.emitln(&format!("\n{ih}:"));
        let jv = self.fresh_tmp(); self.emitln(&format!("  {jv} = load i64, i64* {js}"));
        let jo = self.fresh_tmp(); self.emitln(&format!("  {jo} = icmp sge i64 {jv}, 0"));
        self.emitln(&format!("  br i1 {jo}, label %{ib}, label %{id}"));
        self.emitln(&format!("\n{ib}:"));
        let joff = self.fresh_tmp(); self.emitln(&format!("  {joff} = mul i64 {jv}, {eszv}"));
        let jptr = self.fresh_tmp(); self.emitln(&format!("  {jptr} = getelementptr i8, i8* {dp}, i64 {joff}"));
        let jpi = self.fresh_tmp(); self.emitln(&format!("  {jpi} = bitcast i8* {jptr} to i64*"));
        let je = self.fresh_tmp(); self.emitln(&format!("  {je} = load i64, i64* {jpi}"));
        let cmp = self.fresh_tmp(); self.emitln(&format!("  {cmp} = icmp sgt i64 {je}, {kv}"));
        let shift = self.fresh_block("sort_shift");
        self.emitln(&format!("  br i1 {cmp}, label %{shift}, label %{id}"));
        self.emitln(&format!("\n{shift}:"));
        let jp1 = self.fresh_tmp(); self.emitln(&format!("  {jp1} = add i64 {jv}, 1"));
        let jp1o = self.fresh_tmp(); self.emitln(&format!("  {jp1o} = mul i64 {jp1}, {eszv}"));
        let jp1p = self.fresh_tmp(); self.emitln(&format!("  {jp1p} = getelementptr i8, i8* {dp}, i64 {jp1o}"));
        let jp1pi = self.fresh_tmp(); self.emitln(&format!("  {jp1pi} = bitcast i8* {jp1p} to i64*"));
        self.emitln(&format!("  store i64 {je}, i64* {jp1pi}"));
        let jd = self.fresh_tmp(); self.emitln(&format!("  {jd} = sub i64 {jv}, 1"));
        self.emitln(&format!("  store i64 {jd}, i64* {js}"));
        self.emitln(&format!("  br label %{ih}"));
        self.emitln(&format!("\n{id}:"));
        let jd2 = self.fresh_tmp(); self.emitln(&format!("  {jd2} = load i64, i64* {js}"));
        let jd21 = self.fresh_tmp(); self.emitln(&format!("  {jd21} = add i64 {jd2}, 1"));
        let jd21o = self.fresh_tmp(); self.emitln(&format!("  {jd21o} = mul i64 {jd21}, {eszv}"));
        let jd21p = self.fresh_tmp(); self.emitln(&format!("  {jd21p} = getelementptr i8, i8* {dp}, i64 {jd21o}"));
        let jd21pi = self.fresh_tmp(); self.emitln(&format!("  {jd21pi} = bitcast i8* {jd21p} to i64*"));
        self.emitln(&format!("  store i64 {kv}, i64* {jd21pi}"));
        let ii = self.fresh_tmp(); self.emitln(&format!("  {ii} = add i64 {iv}, 1"));
        self.emitln(&format!("  store i64 {ii}, i64* {i_slot}"));
        self.emitln(&format!("  br label %{oh}"));
        self.emitln(&format!("\n{od}:"));
        self.emitln(&format!("  br label %{skip}"));
        self.emitln(&format!("\n{skip}:"));
        let vec_back = self.emit_vec_load_fields(&vec_alloca);
        self.store_back_to_receiver(receiver, &vec_back, "%struct.Vec");
        Ok(())
    }
}
