use super::IrEmitter;
use xiom_ast::*;
// B-001: llvm_consts not currently used here, but kept for future
// concrete type lowering that will need LLVM_STR_PTR etc.
#[allow(unused_imports)]
use crate::llvm_consts::*;

impl IrEmitter {
    pub(crate) fn coerce_arg_for_param(&mut self, arg_expr: &Expr, pre_val: &str, pre_ty: &str, param_ty: &str) -> String {
        // round-15 (probe_map): a BY-VALUE [N]T param (array.map/zip) receiving
        // a %struct.Vec VALUE (unannotated VAR array literals convert to Vec
        // -- M33) -- the mono def declares "[5 x i64]" and reads the aggregate,
        // so the caller must MATERIALIZE it from the Vec's data buffer
        // (element i at data + i*elem_size). The old pass-through fed the Vec
        // HEADER bits as the array -> garbage/AV (smoke_array_map exit 1).
        if param_ty.starts_with('[') && param_ty.contains(" x ") && !param_ty.ends_with('*') {
            if let (Some(n), elem_llvm) = (Self::extract_array_len(param_ty), Self::extract_array_elem_ty(param_ty)) {
                if pre_ty == "%struct.Vec" && n > 0 {
                    let v_alloca = self.fresh_tmp();
                    self.emitln(&format!("  {v_alloca} = alloca %struct.Vec"));
                    self.emitln(&format!("  store %struct.Vec {pre_val}, %struct.Vec* {v_alloca}"));
                    let dp_gep = self.fresh_tmp();
                    self.emitln(&format!("  {dp_gep} = getelementptr %struct.Vec, %struct.Vec* {v_alloca}, i32 0, i32 0"));
                    let data = self.fresh_tmp();
                    self.emitln(&format!("  {data} = load i8*, i8** {dp_gep}"));
                    let esz_gep = self.fresh_tmp();
                    self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, %struct.Vec* {v_alloca}, i32 0, i32 3"));
                    let esz = self.fresh_tmp();
                    self.emitln(&format!("  {esz} = load i64, i64* {esz_gep}"));
                    let agg = self.fresh_tmp();
                    self.emitln(&format!("  {agg} = alloca {param_ty}"));
                    for i in 0..n {
                        let off = self.fresh_tmp();
                        self.emitln(&format!("  {off} = mul i64 {i}, {esz}"));
                        let ep = self.fresh_tmp();
                        self.emitln(&format!("  {ep} = getelementptr i8, i8* {data}, i64 {off}"));
                        let eptr = self.fresh_tmp();
                        self.emitln(&format!("  {eptr} = bitcast i8* {ep} to {elem_llvm}*"));
                        let ev = self.fresh_tmp();
                        self.emitln(&format!("  {ev} = load {elem_llvm}, {elem_llvm}* {eptr}"));
                        let agep = self.fresh_tmp();
                        self.emitln(&format!("  {agep} = getelementptr {param_ty}, {param_ty}* {agg}, i64 0, i64 {i}"));
                        self.emitln(&format!("  store {elem_llvm} {ev}, {elem_llvm}* {agep}"));
                    }
                    let loaded = self.fresh_tmp();
                    self.emitln(&format!("  {loaded} = load {param_ty}, {param_ty}* {agg}"));
                    return loaded;
                }
            }
        }
        // By-value struct params: `&Vec[T]`/`&Slice[T]` receive the STRUCT value
        // (%struct.Vec), NOT the address. For a `Ref(lvalue)` arg, compile the
        // inner value directly -- otherwise the caller passes the Vec's data
        // pointer (i8*) which coerces via inttoptr+load into a GARBAGE struct
        // (reading the element buffer as the Vec header -- ACCESS_VIOLATION).
        // Scalar `&T` params are EXCLUDED: they carry the ADDRESS as i64
        // (the &literal/&local temp path in expr.rs supplies it).
        // Pointer params (`%struct.Vec2*` for &mut Vec2) are also EXCLUDED --
        // they must receive the lvalue's slot ADDRESS so mutations propagate
        // (m33_b18: `set_x(&mut p, 3)` writes through the caller's alloca).
        if param_ty.starts_with("%struct.") && !param_ty.ends_with('*') {
            // LET-array P3 (docs/LET_ARRAY_DECISION.md): a FIXED-array
            // argument (`let a = [...]` -> `[N x T]`, annotated var arrays,
            // or a `&[N]T` element-pointer param) passed to a by-value
            // Slice/Vec param (`&Slice[T]` and `Vec[T]` both lower to
            // %struct.Vec) materializes a heap-backed Vec with the array's
            // elements (len = N, cap = max(N,16), elem_size = size_of(T)).
            // Without this bridge the collection API (`core.is_sorted(a)`,
            // `slice.*`, by-value `Vec` params) would receive the raw array
            // VALUE / stack view where it expects a Vec header.
            if Self::is_llvm_struct_named(param_ty, "Vec") || Self::is_llvm_struct_named(param_ty, "Slice") {
                if let Some(view) = self.array_as_vec_arg(arg_expr) {
                    return view;
                }
            }
            if let Expr::Ref(i, _) | Expr::MutRef(i, _) = arg_expr {
                if let Ok((v, t)) = self.compile_expr(i) {
                    return self.coerce_value(&v, &t, param_ty);
                }
            }
        }
        if param_ty.ends_with('*') {
            // LET-array P3: a FIXED-array arg passed to a pointer-to-Vec param
            // (`&Vec[T]` -> `%struct.Vec*`) must receive a REAL header (the
            // callee reads len/cap/elem_size). Synthesize the Slice view into
            // an alloca and pass its address; the old path bitcast the array
            // slot to %struct.Vec* and the callee read element bytes as the
            // header (test_algo binary_search AV). Only for array bindings --
            // real Vec locals keep the existing copy/address path.
            if Self::is_llvm_struct_named(param_ty, "Vec") || Self::is_llvm_struct_named(param_ty, "Slice") {
                if let Some(view) = self.array_as_vec_arg(arg_expr) {
                    let slot = self.fresh_tmp();
                    self.emitln(&format!("  {slot} = alloca %struct.Vec"));
                    self.emitln(&format!("  store %struct.Vec {view}, %struct.Vec* {slot}"));
                    return slot;
                }
            }
            // BUG 44: `&T` -> T auto-coercion for POINTER-pointee types (Str).
            // An arg carrying an ADDRESS (`&s`, or a ref-local/ref-param
            // ident `p: &Str`) must be DEREF'd when the param expects the
            // pointee VALUE (`expect_str(&s)` -- the old code passed the slot
            // ADDRESS, or worse truncated it to a byte). Scalar `&T` -> T is
            // ambiguous with `&T` params at this layer (both lower to "i64")
            // and keeps the established address-passthrough; `i8**` params
            // (&Str) want the address itself.
            if !param_ty.ends_with("**") {
                if let Some(pointee_arg) = self.coerce_ref_arg_to_pointee(arg_expr, &param_ty) {
                    return pointee_arg;
                }
            }
            let lvalue: Option<&Expr> = match arg_expr {
                Expr::Ref(i, _) | Expr::MutRef(i, _) => Some(i.as_ref()),
                Expr::Unary(UnaryOp::Ref, i, _) | Expr::Unary(UnaryOp::MutRef, i, _) => Some(i.as_ref()),
                _ => None,
            };
            if let Some(Expr::Ident(id)) = lvalue {
                // &array_local -> pass the Vec's DATA pointer (field 0), not the
                // Vec alloca address. The `&[N]T` callee indexes the data buffer.
                // BUG 30: ONLY for fixed-array-style params (i64*, [N x T]*).
                // A `%struct.Vec*` param (&Vec[T]) must receive the HEADER
                // alloca address -- the data-pointer path made len() read the
                // first ELEMENT as the length -> OOB -> AV (eco test_algo).
                if self.local.array_locals.contains(&id.name) && !param_ty.contains("%struct.") {
                    if let Some((slot, slot_ty)) = self.lookup_local(&id.name).cloned() {
                        // BUG 53 (2026-08-18): FIXED-ARRAY slots (`var a:
                        // [5]Int = [...]` -> slot `[5 x i64]`) -- the element
                        // pointer is GEP 0,0 of the array; the old code GEP'd
                        // the slot AS %struct.Vec and loaded "field 0" (the
                        // first ELEMENT, e.g. 10) as the data pointer ->
                        // garbage reads in every &[N]T fn.
                        if slot_ty.starts_with('[') && slot_ty.contains(" x ") {
                            let elem_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {elem_ptr} = getelementptr {slot_ty}, {slot_ty}* {slot}, i64 0, i64 0"));
                            let elem_llvm = Self::extract_array_elem_ty(&slot_ty);
                            return self.coerce_value(&elem_ptr, &format!("{elem_llvm}*"), param_ty);
                        }
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr %struct.Vec, %struct.Vec* {slot}, i32 0, i32 0"));
                        let data_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {data_ptr} = load i8*, i8** {gep}"));
                        return self.coerce_value(&data_ptr, "i8*", param_ty);
                    }
                }
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
            // NOTE: non-ident `&expr` lvalues (Index/Field/Call) MUST NOT be
            // re-compiled here -- the Expr::Ref/MutRef codegen arm already
            // lowers them (Index -> element ADDRESS as i64, Field -> GEP pointer,
            // Call -> value). Falling through to coerce_value preserves that:
            // an i64 address inttoptrs to the real pointer, a struct value
            // takes the existing struct->pointer slot path (coerce_value),
            // and a Vec data pointer coerces directly.
            // BUG 52 (2026-08-18): `key: &K` with K=Str -- the mono param is
            // `i8**` (address OF a slot holding the string handle). A Str
            // VALUE arg (`m.get("b")`, a Vec[Str] element) must be
            // MATERIALIZED into an i8* slot so the callee's `*key` loads the
            // handle -- passing the handle itself as i8** made `*key` read
            // the string's first 8 BYTES as a pointer (garbage -> strcmp AV;
            // Map.get with Str keys crashed for Int AND enum values alike).
            // Arg forms covered: a plain Str value AND `&"literal"` / `&(expr)`
            // (the Ref arm yields the i8* literal value, pre_ty i8*). Ref-LOCALS
            // and `&str_local` already carry the ADDRESS (pre_ty i8**) and skip
            // this. The old `lvalue.is_none()` gate let `&"password"` through
            // as a BITCAST of the string bytes -- crypto.pbkdf2 then read the
            // literal's first 8 bytes as the Str handle (str_len AV at -1;
            // the standalone replica with a local password worked).
            if param_ty == "i8**" && pre_ty == "i8*" {
                let tmp = self.fresh_tmp();
                self.emitln(&format!("  {tmp} = alloca i8*"));
                self.emitln(&format!("  store i8* {pre_val}, i8** {tmp}"));
                return tmp;
            }
            if lvalue.is_none() && !pre_ty.ends_with('*') && !param_ty.starts_with("%struct.") {
                // BUG 31: a plain VALUE arg (literal etc.) to a `&T`/pointer
                // param must be MATERIALIZED into a temp -- the inttoptr
                // fallback treated the VALUE as an address
                // (Map.get(1) -> callee's `*key` derefs address 1 -> AV).
                // Only for scalar pointees (i64*/i8*/double*); array and
                // struct-pointer params keep their existing paths.
                // BUG 37/36 follow-up (2026-08-17): a Str-typed arg in i64
                // form (Vec[Str] element read, Option payload, etc.) is a
                // STRING HANDLE, not a byte value -- materializing it as a
                // single-byte temp printed garbage (smoke_serialize yaml
                // sequence). The i64 handle must inttoptr to i8* instead.
                let pointee = param_ty.trim_end_matches('*');
                if matches!(pointee, "i1" | "i8" | "i16" | "i32" | "i64" | "float" | "double" | "fp128") {
                    // Args whose XIOM type is Str (string handle) or a raw
                    // pointer (`*T` -- e.g. `var p = ptr.null[Int]()`) carry
                    // POINTER BITS in i64 form; they must inttoptr to the
                    // param type, not materialize a single-element temp
                    // (smoke_serialize yaml sequence; smoke_ptr is_null).
                    let arg_xiom = if let Expr::Ident(id) = arg_expr {
                        self.xiom_type_of_local(&id.name)
                    } else {
                        // Stack-cookie family (2026-09-10): a CALL returning a
                        // raw pointer carries POINTER BITS in i64 form, but
                        // infer_value_xiom_type does not resolve call returns --
                        // such an arg was "materialized" as a single i8 (trunc +
                        // 1-byte alloca) and passed as a buffer DEST with a
                        // 4096-byte size -> stack overrun (0xC0000409). Resolve
                        // the callee's declared return so pointer-valued calls
                        // inttoptr like pointer-valued locals.
                        // R7 (2026-09-10): an INDEX arg (`entries.keys[i]`) is an
                        // element VALUE -- resolve the container's element type
                        // ("Str") so the handle inttoptrs instead of truncating
                        // to a byte (json stringify keys printed as garbage).
                        Self::infer_value_xiom_type(arg_expr)
                            .or_else(|| self.infer_call_return_xiom(arg_expr))
                            .or_else(|| match arg_expr {
                                Expr::Index(container, _, _) => self.resolve_vec_elem_xiom(container),
                                _ => None,
                            })
                    };
                    let arg_is_ptr_valued = arg_xiom.as_deref().map_or(false, |t| {
                        t == "Str" || t.starts_with('*')
                    });
                    if !arg_is_ptr_valued {
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = alloca {pointee}"));
                        let cv = self.coerce_value(pre_val, pre_ty, pointee);
                        self.emitln(&format!("  store {pointee} {cv}, {pointee}* {tmp}"));
                        return tmp;
                    }
                }
            }
        }
        self.coerce_value(pre_val, pre_ty, param_ty)
    }

    /// LET-array P3 (docs/LET_ARRAY_DECISION.md): materialize a heap-backed
    /// `%struct.Vec` from a FIXED-array argument. Accepts `a`, `&a`, `&mut a`
    /// (and parens) for a `[N x T]` local or a `&[N]T` element-pointer param
    /// with a known const N. Returns None when the arg is not such a binding.
    ///
    /// The elements are COPIED to the heap (cap = max(N, 16)), matching the
    /// pre-P3 M33 representation the callers were built against: a by-value
    /// `Vec[T]` param may PUSH, and realloc on a stack view corrupts the heap
    /// (test_algo concat: `var result = a; result.push(...)`). Read-only
    /// `&Slice[T]` consumers are equally happy with the heap backing.
    fn array_as_vec_arg(&mut self, arg_expr: &Expr) -> Option<String> {
        let mut inner: &Expr = arg_expr;
        loop {
            match inner {
                Expr::Paren(p, _) => inner = p.as_ref(),
                Expr::Ref(i, _) | Expr::MutRef(i, _)
                | Expr::Unary(UnaryOp::Ref, i, _) | Expr::Unary(UnaryOp::MutRef, i, _) => {
                    inner = i.as_ref();
                }
                _ => break,
            }
        }
        let Expr::Ident(id) = inner else { return None; };
        let n = self.local.local_array_sizes.get(&id.name).copied()?;
        if n <= 0 { return None; }
        let (slot, slot_ty) = self.lookup_local(&id.name).cloned()?;
        let (base, elem_llvm) = if slot_ty.starts_with('[') && slot_ty.contains(" x ") && !slot_ty.ends_with('*') {
            // `[N x T]` local: GEP to the first element.
            let elem = Self::extract_array_elem_ty(&slot_ty);
            let ep = self.fresh_tmp();
            self.emitln(&format!("  {ep} = getelementptr {slot_ty}, {slot_ty}* {slot}, i64 0, i64 0"));
            (ep, elem)
        } else if self.is_array_elem_param(inner) && slot_ty.ends_with('*') {
            // `&[N]T` param: the slot holds the element pointer.
            let elem = self.local.local_array_elem.get(&id.name).cloned()?;
            let loaded = self.fresh_tmp();
            self.emitln(&format!("  {loaded} = load {slot_ty}, {slot_ty}* {slot}"));
            (loaded, elem)
        } else {
            return None;
        };
        let elem_size = Self::llvm_type_byte_size(&elem_llvm, &self.types.type_meta) as i64;
        let cap = n.max(16);
        let src = self.fresh_tmp();
        self.emitln(&format!("  {src} = bitcast {elem_llvm}* {base} to i8*"));
        let data = self.fresh_tmp();
        self.emitln(&format!("  {data} = call i8* @malloc(i64 {})", cap * elem_size));
        let null_check = self.fresh_tmp();
        let ok_block = self.fresh_block("arr2vec_ok");
        let trap_block = self.fresh_block("arr2vec_trap");
        self.emitln(&format!("  {null_check} = icmp eq i8* {data}, null"));
        self.emitln(&format!("  br i1 {null_check}, label %{trap_block}, label %{ok_block}"));
        self.emitln(&format!("\n{trap_block}:"));
        self.emitln("  call void @llvm.trap()");
        self.emitln("  unreachable");
        self.emitln(&format!("\n{ok_block}:"));
        let bytes = self.fresh_tmp();
        self.emitln(&format!("  {bytes} = mul i64 {n}, {elem_size}"));
        self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {data}, i8* {src}, i64 {bytes}, i1 false)"));
        let va = self.fresh_tmp();
        self.emitln(&format!("  {va} = alloca %struct.Vec"));
        let dg = self.fresh_tmp();
        self.emitln(&format!("  {dg} = getelementptr %struct.Vec, %struct.Vec* {va}, i32 0, i32 0"));
        self.emitln(&format!("  store i8* {data}, i8** {dg}"));
        let lg = self.fresh_tmp();
        self.emitln(&format!("  {lg} = getelementptr %struct.Vec, %struct.Vec* {va}, i32 0, i32 1"));
        self.emitln(&format!("  store i64 {n}, i64* {lg}"));
        let cg = self.fresh_tmp();
        self.emitln(&format!("  {cg} = getelementptr %struct.Vec, %struct.Vec* {va}, i32 0, i32 2"));
        self.emitln(&format!("  store i64 {cap}, i64* {cg}"));
        let eg = self.fresh_tmp();
        self.emitln(&format!("  {eg} = getelementptr %struct.Vec, %struct.Vec* {va}, i32 0, i32 3"));
        self.emitln(&format!("  store i64 {elem_size}, i64* {eg}"));
        let loaded = self.fresh_tmp();
        self.emitln(&format!("  {loaded} = load %struct.Vec, %struct.Vec* {va}"));
        Some(loaded)
    }

    /// BUG 44: when a call arg carries an ADDRESS but the param expects the
    /// pointee VALUE (auto-coercion `&Str` -> `Str`), deref the address and
    /// return the loaded pointee value. Handles both `&ident` REF EXPR args
    /// and ref-local/ref-param IDENT args (`p: &Str` passed to a `Str` param).
    /// Only fires when the param type EXACTLY equals the pointee's LLVM type
    /// (i8* for Str) -- `&T` params (i64 / i8**) keep receiving the address.
    fn coerce_ref_arg_to_pointee(&mut self, arg_expr: &Expr, param_ty: &str) -> Option<String> {
        match arg_expr {
            // `&s` / `&mut s`: compile the INNER lvalue -- its value IS the
            // pointee. (`f(&x)` to a `&T` param never reaches here: `&T`
            // params are i64/i8** and the inner value type won't match.)
            Expr::Ref(inner, _) | Expr::MutRef(inner, _) => {
                if let Ok((iv, it)) = self.compile_expr(inner) {
                    if param_ty == it {
                        return Some(iv);
                    }
                }
                None
            }
            // `p` where p is a ref-local/ref-param (`var p = &s`, `s: &Str`):
            // the ident holds the ADDRESS -- inttoptr (if i64) + load.
            Expr::Ident(id) => {
                let is_ref = self.local.ref_params.contains(&id.name)
                    || self.local.ref_locals.contains(&id.name);
                if !is_ref { return None; }
                // BUG 55 (2026-08-18): RAW-POINTER locals (`var p: *Int = &x`,
                // `p: *Str`) are POINTER-VALUED -- the recorded XIOM type is the
                // raw pointer ("*Int" -> i64*). Passing `p` to a `*Int` param
                // must pass the pointer ITSELF; the old code treated every
                // ref-tracked local as address-carrying and DEREF'd it
                // (`%tmp9 = load i64*, i64** %tmp8` -- x's VALUE became the
                // pointer arg -> null/garbage at the callee -> AV/wrong reads
                // across fn/unsafe-block boundaries; the payload-corruption
                // family root). The auto-deref applies ONLY to `&T` ref-locals
                // whose recorded type is the bare pointee ("Int"/"Str").
                let pointee_xiom = self.local.local_xiom_types.get(&id.name)?;
                if pointee_xiom.starts_with('*') { return None; }
                let pointee = self.llvm_type_for(pointee_xiom).ok()?;
                if pointee != param_ty { return None; }
                if let Ok((addr, addr_ty)) = self.compile_expr(arg_expr) {
                    let ptr = if addr_ty == "i64" {
                        let p = self.fresh_tmp();
                        self.emitln(&format!("  {p} = inttoptr i64 {addr} to {pointee}*"));
                        p
                    } else {
                        addr
                    };
                    let loaded = self.fresh_tmp();
                    self.emitln(&format!("  {loaded} = load {pointee}, {pointee}* {ptr}"));
                    return Some(loaded);
                }
                None
            }
            _ => None,
        }
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
                // M17: Use reg_signed tracking for per-register signedness, falling
                // back to type-based defaults (zext for i1/i8, sext for i16/i32).
                // This matches the logic in widen_to_i64.
                let is_signed = self.local.reg_signed.get(val).copied().unwrap_or_else(|| {
                    // Default: sext for i8/i16/i32, zext for i1 (Bool).
                    !matches!(from, "i1")
                });
                let op = if is_signed { "sext" } else { "zext" };
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
        // BUG 13 fix: Float128 (fp128) coercions -- the As-cast handler had
        // fp128 arms but coerce_value (stores/params/returns) had none, so
        // `var n: Float128 = 5` emitted `store fp128 5` (clang rejects the
        // integer constant) and fp128 flows silently passed values through.
        if from == "i64" && to == "fp128" {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = sitofp i64 {val} to fp128"));
            return t;
        }
        if from == "fp128" && to == "i64" {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = fptosi fp128 {val} to i64"));
            return t;
        }
        if (from == "double" || from == "float") && to == "fp128" {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = fpext {from} {val} to fp128"));
            return t;
        }
        if from == "fp128" && (to == "double" || to == "float") {
            let t = self.fresh_tmp();
            self.emitln(&format!("  {t} = fptrunc fp128 {val} to {to}"));
            return t;
        }
        // 5c-E: i8* buffer pointer -> %struct.Vec coercion.
        // Used when returning a Vec-typed array literal from a function
        // (e.g. `return out` where `out: Vec[Float32] = []` is i8*).
        if from == "i8*" && to == "%struct.Vec" {
            return self.val_to_struct(val, from, to);
        }
        // Concrete container -> OPAQUE base (m21_struct_mut_015): a generic
        // fn param `Option[T]` can mono to the erased `%struct.Option` while
        // the caller built Option__Data (registered by the ctor/pre-pass).
        // Rebuild the erased shape: copy the tag and BOX each struct payload
        // (the opaque ABI stores an i64 handle; scalar payloads copy bits).
        if from.starts_with("%struct.") && from != to
            && (to == "%struct.Option" || to == "%struct.Result")
            && (from.contains("Option__") || from.contains("Result__"))
        {
            let from_name = from.trim_start_matches("%struct.").to_string();
            let from_fields: Vec<String> = self.types.type_meta.get(&from_name)
                .map(|m| m.fields.iter().map(|(_, t)| t.clone()).collect())
                .unwrap_or_default();
            if !from_fields.is_empty() {
                let src = self.fresh_tmp();
                self.emitln(&format!("  {src} = alloca {from}"));
                self.emitln(&format!("  store {from} {val}, {from}* {src}"));
                let dst = self.fresh_tmp();
                self.emitln(&format!("  {dst} = alloca {to}"));
                self.emitln(&format!("  store {to} zeroinitializer, {to}* {dst}"));
                let tag = self.fresh_tmp();
                let sg0 = self.fresh_tmp();
                self.emitln(&format!("  {sg0} = getelementptr {from}, {from}* {src}, i32 0, i32 0"));
                self.emitln(&format!("  {tag} = load i64, i64* {sg0}"));
                let dg0 = self.fresh_tmp();
                self.emitln(&format!("  {dg0} = getelementptr {to}, {to}* {dst}, i32 0, i32 0"));
                self.emitln(&format!("  store i64 {tag}, i64* {dg0}"));
                for i in 1..from_fields.len() {
                    let fty = self.field_llvm_ty(&from_fields[i]);
                    let sg = self.fresh_tmp();
                    self.emitln(&format!("  {sg} = getelementptr {from}, {from}* {src}, i32 0, i32 {i}"));
                    let dg = self.fresh_tmp();
                    self.emitln(&format!("  {dg} = getelementptr {to}, {to}* {dst}, i32 0, i32 {i}"));
                    let raw = self.fresh_tmp();
                    self.emitln(&format!("  {raw} = load {fty}, {fty}* {sg}"));
                    let as_i64 = if fty.starts_with("%struct.") {
                        // Box the inline payload: the erased slot holds a pointer.
                        let slot = self.fresh_tmp();
                        self.emitln(&format!("  {slot} = alloca {fty}"));
                        self.emitln(&format!("  store {fty} {raw}, {fty}* {slot}"));
                        let p = self.fresh_tmp();
                        self.emitln(&format!("  {p} = ptrtoint {fty}* {slot} to i64"));
                        p
                    } else if fty.ends_with('*') {
                        let p = self.fresh_tmp();
                        self.emitln(&format!("  {p} = ptrtoint {fty} {raw} to i64"));
                        p
                    } else if fty == "double" {
                        let b = self.fresh_tmp();
                        self.emitln(&format!("  {b} = bitcast double {raw} to i64"));
                        b
                    } else if fty == "float" {
                        let b = self.fresh_tmp();
                        self.emitln(&format!("  {b} = bitcast float {raw} to i32"));
                        let w = self.fresh_tmp();
                        self.emitln(&format!("  {w} = zext i32 {b} to i64"));
                        w
                    } else {
                        self.coerce_value(&raw, &fty, "i64")
                    };
                    self.emitln(&format!("  store i64 {as_i64}, i64* {dg}"));
                }
                let out = self.fresh_tmp();
                self.emitln(&format!("  {out} = load {to}, {to}* {dst}"));
                return out;
            }
        }
        // Opaque container -> concrete container (M65 R7 / BUG 41 follow-on):
        // a value built while the expected type was the ERASED base
        // (`%struct.Result` from an Ok(...) ctor in a fn NOT returning
        // Result) stored into a concretely-annotated slot
        // (`var r: Result[Payload, Int]`). The opaque slot keeps an i64
        // payload (boxed pointer for struct payloads, raw bits otherwise);
        // the concrete struct expects the real field type. Copy the tag and
        // rebuild every payload field through the same i64->field rules the
        // payload override uses.
        if from.starts_with("%struct.") && to.starts_with("%struct.") && from != to
            && (to.contains("Result__") || to.contains("Option__"))
        {
            let from_name = from.trim_start_matches("%struct.").to_string();
            let to_name = to.trim_start_matches("%struct.").to_string();
            let from_field_count = self.types.types.get(&from_name).map(|v| v.len()).unwrap_or(0);
            let to_fields: Vec<String> = self.types.type_meta.get(&to_name)
                .map(|m| m.fields.iter().map(|(_, t)| t.clone()).collect())
                .unwrap_or_default();
            if !to_fields.is_empty() {
                let src = self.fresh_tmp();
                self.emitln(&format!("  {src} = alloca {from}"));
                self.emitln(&format!("  store {from} {val}, {from}* {src}"));
                let dst = self.fresh_tmp();
                self.emitln(&format!("  {dst} = alloca {to}, align 16"));
                let tag = self.fresh_tmp();
                let sg0 = self.fresh_tmp();
                self.emitln(&format!("  {sg0} = getelementptr {from}, {from}* {src}, i32 0, i32 0"));
                self.emitln(&format!("  {tag} = load i64, i64* {sg0}"));
                let dg0 = self.fresh_tmp();
                self.emitln(&format!("  {dg0} = getelementptr {to}, {to}* {dst}, i32 0, i32 0"));
                self.emitln(&format!("  store i64 {tag}, i64* {dg0}"));
                // Payload fields: only the ACTIVE variant's slot holds a value
                // (None/Err store 0) -- dereferencing the inactive slot read
                // address 0 (m21_result_option_013 / m35_o06 traps). Guard each
                // payload conversion by its tag branch.
                for i in 1..to_fields.len() {
                    let dg = self.fresh_tmp();
                    self.emitln(&format!("  {dg} = getelementptr {to}, {to}* {dst}, i32 0, i32 {i}"));
                    let target = self.field_llvm_ty(&to_fields[i]);
                    if i >= from_field_count {
                        continue;
                    }
                    let sg = self.fresh_tmp();
                    self.emitln(&format!("  {sg} = getelementptr {from}, {from}* {src}, i32 0, i32 {i}"));
                    let raw = self.fresh_tmp();
                    self.emitln(&format!("  {raw} = load i64, i64* {sg}"));
                    // Some/Ok = tag 1; Result Err = tag 0 (the ctor stores
                    // discriminant 0 for Err -- see the m21_039 Err arm). Option
                    // has no tag-0 payload, so the Result-only condition is
                    // unreachable there.
                    let active_tag: i64 = if i == 1 { 1 } else { 0 };
                    let act = self.fresh_tmp();
                    self.emitln(&format!("  {act} = icmp eq i64 {tag}, {active_tag}"));
                    let on_b = self.fresh_block("ctr_on");
                    let off_b = self.fresh_block("ctr_off");
                    let merge_b = self.fresh_block("ctr_merge");
                    self.emitln(&format!("  br i1 {act}, label %{on_b}, label %{off_b}"));
                    self.emitln(&format!("\n{on_b}:"));
                    let conv = if target.starts_with("%struct.") {
                        let sp = self.fresh_tmp();
                        self.emitln(&format!("  {sp} = inttoptr i64 {raw} to {target}*"));
                        let sv = self.fresh_tmp();
                        self.emitln(&format!("  {sv} = load {target}, {target}* {sp}"));
                        sv
                    } else if target.ends_with('*') {
                        let sp = self.fresh_tmp();
                        self.emitln(&format!("  {sp} = inttoptr i64 {raw} to {target}"));
                        sp
                    } else if target == "double" {
                        let bv = self.fresh_tmp();
                        self.emitln(&format!("  {bv} = bitcast i64 {raw} to double"));
                        bv
                    } else if target == "float" {
                        let t32 = self.fresh_tmp();
                        self.emitln(&format!("  {t32} = trunc i64 {raw} to i32"));
                        let bv = self.fresh_tmp();
                        self.emitln(&format!("  {bv} = bitcast i32 {t32} to float"));
                        bv
                    } else {
                        self.coerce_value(&raw, "i64", &target)
                    };
                    self.emitln(&format!("  br label %{merge_b}"));
                    self.emitln(&format!("\n{off_b}:"));
                    self.emitln(&format!("  br label %{merge_b}"));
                    self.emitln(&format!("\n{merge_b}:"));
                    let default_const = if target.starts_with("%struct.") {
                        "zeroinitializer".to_string()
                    } else if target.ends_with('*') {
                        "null".to_string()
                    } else if target == "double" || target == "float" {
                        "0.0".to_string()
                    } else {
                        "0".to_string()
                    };
                    let phi = self.fresh_tmp();
                    self.emitln(&format!(
                        "  {phi} = phi {target} [ {conv}, %{on_b} ], [ {default_const}, %{off_b} ]"
                    ));
                    self.emitln(&format!("  store {target} {phi}, {target}* {dg}"));
                }
                let out = self.fresh_tmp();
                self.emitln(&format!("  {out} = load {to}, {to}* {dst}"));
                return out;
            }
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
        // No known cast -- return unchanged (best effort).
        val.to_string()
    }

    /// Extract field 0 (the leading scalar -- e.g. an enum discriminant or an
    /// Option/Result's first slot) from a by-value struct `val` of type
    /// `struct_ty`, returning the loaded `i64` scalar register. Used when a
    /// single-scalar-backed struct value appears in an integer context (e.g.
    /// `opt >= 0`). Returns `val` unchanged when `struct_ty` isn't a struct, and
    /// a `0` constant for empty (zero-field) structs which have no field 0.
    pub(crate) fn extract_scalar_field0(&mut self, val: &str, struct_ty: &str) -> String {
        if !struct_ty.starts_with("%struct.") {
            return val.to_string();
        }
        let type_name = &struct_ty[8..];
        // AUDIT BUG 57 follow-up: a Vec/Slice value must NEVER take the
        // scalar field-0 extraction -- field 0 is the DATA POINTER (i8*);
        // loading it as i64 produced invalid IR ("%tmpN defined with type
        // i64 but expected %struct.Vec"; geom_vec/mat compile errors).
        // Load it AS the pointer and ptrtoint: VALID IR with the same
        // value semantics the pre-map codegen had (an i64 operand whose
        // bits are the data pointer). The upstream root (a chained-index
        // read yielding a Vec where a double was expected) is tracked as
        // the BUG 57-geom follow-up.
        let is_vecish = type_name == "Vec" || type_name == "Slice"
            || type_name.ends_with(".Vec") || type_name.ends_with(".Slice");
        if is_vecish {
            let slot = self.fresh_tmp();
            self.emitln(&format!("  {slot} = alloca {struct_ty}"));
            self.emitln(&format!("  store {struct_ty} {val}, {struct_ty}* {slot}"));
            let gep = self.fresh_tmp();
            self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {slot}, i32 0, i32 0"));
            let ptr = self.fresh_tmp();
            self.emitln(&format!("  {ptr} = load i8*, i8** {gep}"));
            let as_i64 = self.fresh_tmp();
            self.emitln(&format!("  {as_i64} = ptrtoint i8* {ptr} to i64"));
            return as_i64;
        }
        // Empty (zero-sized) structs have no field 0 -- GEP would be invalid.
        let is_empty = self.types.type_meta.get(&type_name.to_string()).map(|m| m.fields.is_empty()).unwrap_or(false);
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
        let is_empty = self.types.type_meta.get(&type_name.to_string()).map(|m| m.fields.is_empty()).unwrap_or(false);
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
        } else if ty == "i1" || ty == "i8" || ty == "i16" || ty == "i32" {
            // M17: Use reg_signed tracking for per-register signedness, falling
            // back to type-based defaults (zext for i1/i8, sext for i16/i32).
            let is_signed = self.local.reg_signed.get(val).copied().unwrap_or_else(|| {
                !matches!(ty, "i1")
            });
            let op = if is_signed { "sext" } else { "zext" };
            let ext = self.fresh_tmp();
            self.emitln(&format!("  {ext} = {op} {ty} {val} to i64"));
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
            let malloc_ptr = self.emit_alloc(&size_i64);
            let typed_ptr = self.fresh_tmp();
            let bc = self.fresh_tmp();
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

    /// Concatenation-operand conversion for `Str + X`. Pointer operands bitcast
    /// (or pass through for i8*); INTEGER operands (Int/Int8..UInt128/Char --
    /// `is_int` verdict from the XIOM-level type) are FORMATTED via
    /// @xiom_int_to_string instead of inttoptr, which produced a garbage
    /// pointer and crashed at runtime ("y = " + 42 -> AV). FLOAT operands
    /// (double/float -- BUG 19 fix) are formatted via @xiom_double_to_string
    /// (shortest round-trip; NaN -> "nan", +/-inf -> "inf"/"-inf") -- the previous
    /// inttoptr path bitcast the FP bits to a pointer, crashing or printing
    /// garbage ("c = " + NaN -> AV / sentinel text).
    pub(crate) fn concat_val_to_i8ptr(&mut self, val: &str, ty: &str, is_int: bool) -> String {
        if ty == "i8*" {
            return val.to_string();
        }
        if is_int {
            let i64_val = self.val_to_i64(val, ty);
            let fmt = self.fresh_tmp();
            self.emitln(&format!("  {fmt} = call i8* @xiom_int_to_string(i64 {i64_val})"));
            return fmt;
        }
        if ty == "double" || ty == "float" {
            let (fmt_val, fmt_ty) = if ty == "float" {
                let ext = self.fresh_tmp();
                self.emitln(&format!("  {ext} = fpext float {val} to double"));
                (ext, "double".to_string())
            } else {
                (val.to_string(), "double".to_string())
            };
            let fmt = self.fresh_tmp();
            self.emitln(&format!("  {fmt} = call i8* @xiom_double_to_string({fmt_ty} {fmt_val})"));
            return fmt;
        }
        self.val_to_i8ptr(val, ty)
    }

    /// LLVM scalar integer type (not i1, not a pointer). Used at concat sites
    /// as a FALLBACK verdict when `expr_is_integer` cannot see through an
    /// expression shape (e.g. `e[0]` of a Vec[Int] produced by a chained
    /// `.collect()` -- the local's element registry is absent, so the value
    /// was inttoptr'd into a garbage string pointer: `"E1[0]=" + e[0]` with
    /// e[0]=9 crashed reading address 0x9).
    pub(crate) fn llvm_scalar_is_int(ty: &str) -> bool {
        matches!(ty, "i8" | "i16" | "i32" | "i64" | "i128")
    }

    /// XIOM type name is integer-like (Int*/UInt*/Char).
    fn elem_name_is_int(n: &str) -> bool {
        n == "Int" || n == "Int64" || n == "Char" || n.starts_with("Int") && n.len() <= 6
            || n == "UInt" || n.starts_with("UInt") && n.len() <= 7
    }

    /// The XIOM type of the Vec VALUE an index expression addresses
    /// ("Vec[elem]"). `local_vec_elem` always stores the BARE element type
    /// (params/decls; `Vec.push` records `Vec[inner]` for a Vec-of-Vec's
    /// element, which is also a bare element type), so:
    ///   Ident(v)        -> "Vec[{local_vec_elem[v]}]"
    ///   Index(inner, i) -> element of the inner value's type
    /// This is what lets `rows[0][0]` (Vec[Vec[Str]]) resolve to "Str"
    /// instead of falling through to the concat LLVM-type fallback.
    fn vec_value_xiom_type(&self, e: &Expr) -> Option<String> {
        match e {
            Expr::Ident(id) => self.local.local_vec_elem.get(&id.name)
                .map(|el| format!("Vec[{el}]")),
            Expr::Index(inner, _, _) => {
                let inner_vt = self.vec_value_xiom_type(inner)?;
                Self::element_of_container_type(&inner_vt)
            }
            _ => None,
        }
    }

    /// "Vec[Str]" -> "Str" (also Slice/Array/Map-key containers).
    fn element_of_container_type(vt: &str) -> Option<String> {
        for prefix in ["Vec[", "Slice[", "Array["] {
            if let Some(rest) = vt.strip_prefix(prefix) {
                if let Some(inner) = rest.strip_suffix(']') {
                    return Some(inner.to_string());
                }
            }
        }
        None
    }

    /// True when the indexed element's XIOM type is a CONCRETE non-integer
    /// (Str, Bool, a registered struct, a compound container). Generic
    /// parameters ("T", "U") and unresolved names return false: those stay
    /// "unknown" so the LLVM-type fallback still applies (R14's chained
    /// `.collect()` Vecs).
    fn elem_name_is_known_non_int(&self, n: &str) -> bool {
        if Self::elem_name_is_int(n) {
            return false;
        }
        for p in ["Vec[", "Slice[", "Map[", "Set[", "Stack[", "Option[", "Result[", "Tuple", "fn("] {
            if n.starts_with(p) {
                return true;
            }
        }
        match n {
            "Str" | "Bool" => true,
            _ => {
                let base = n.trim_start_matches('&').trim_start_matches("mut ");
                base.len() > 1
                    && (self.types.types.contains_key(&base.to_string())
                        || self.types.type_meta.contains_key(&base.to_string())
                        || self.types.types.keys().iter().any(|k| k.ends_with(&format!(".{base}")))
                        || self.types.generic_type_names.contains(base))
            }
        }
    }

    /// Concat operand verdict used by BOTH concat sites. Semantic first; the
    /// compiled LLVM scalar fallback only when the semantic walker cannot
    /// tell (unknown Index/Field shapes). Known non-integers (Str elements)
    /// must NOT be formatted as numbers -- they take `val_to_i8ptr`'s
    /// inttoptr, which recovers the pointer bits.
    pub(crate) fn concat_operand_is_int(&self, e: &Expr, llvm_ty: &str) -> bool {
        if self.expr_is_integer(e) {
            return true;
        }
        if let Expr::Index(container, _, _) = e {
            if let Some(vt) = self.vec_value_xiom_type(container) {
                if let Some(el) = Self::element_of_container_type(&vt) {
                    return !self.elem_name_is_known_non_int(&el);
                }
            }
        }
        matches!(e, Expr::Index(..) | Expr::Field(..)) && Self::llvm_scalar_is_int(llvm_ty)
    }

    /// XIOM-level verdict: is this expression an integer (Int*/UInt*/Char)?
    /// Used at concat sites to choose string formatting over inttoptr. Idents
    /// resolve through the registered local type; casts check the target type
    /// name; calls check the callee's registered return type (a real Str
    /// return registers as i8*, so pointer returns are never formatted).
    pub(crate) fn expr_is_integer(&self, e: &Expr) -> bool {
        match e {
            Expr::Int(..) | Expr::Char(..) => true,
            Expr::Ident(id) => {
                // Registered XIOM type (annotation or binding inference) wins.
                if let Some(t) = self.local.local_xiom_types.get(&id.name) {
                    return Self::elem_name_is_int(t);
                }
                // Module-global fallback: the LLVM slot verdict -- a non-pointer
                // integer slot (i64/i32/...) is an Int/UInt global.
                if let Some((_, llvm_ty)) = self.local.module_globals.get(&id.name) {
                    return llvm_ty.starts_with('i') && !llvm_ty.ends_with('*') && llvm_ty != "i1" && llvm_ty != "i8*";
                }
                // Local-slot fallback: derive the XIOM name from the LLVM type.
                self.resolve_local_xiom_type(&id.name)
                    .map(|t| Self::elem_name_is_int(&t))
                    .unwrap_or(false)
            }
            Expr::As(_, ty, _) => Self::elem_name_is_int(&Self::type_from_ast(ty)),
            Expr::Paren(inner, _) => self.expr_is_integer(inner),
            // BUG 22 #11: a parenthesized ARITHMETIC expression of integers is
            // an integer (`"sum = " + (v[0] + v[1])`) -- previously fell through
            // to inttoptr (garbage pointer -> AV).
            Expr::Binary(l, op, r, _) => matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
                && self.expr_is_integer(l) && self.expr_is_integer(r),
            // Vec element reads: the registered element type decides
            // (v[0] of a Vec[Int] is an Int; of a Vec[Float64] is not), and
            // NESTED containers resolve recursively (`rows[0][0]` of a
            // Vec[Vec[Str]] is a Str -- R17 no longer falls through to the
            // concat LLVM fallback).
            Expr::Index(container, _, _) => {
                if let Some(vt) = self.vec_value_xiom_type(container) {
                    if let Some(elem) = Self::element_of_container_type(&vt) {
                        return Self::elem_name_is_int(&elem);
                    }
                }
                false
            }
            Expr::Field(obj, field, _) => {
                // Resolve the object's struct type, then the field's declared
                // XIOM type from type_meta (e.g. `g.v` of a module-global W).
                let struct_name = match self.infer_struct_type_name(obj) {
                    Some(n) => n,
                    None => return false,
                };
                let idx = match self.types.types.get(&struct_name) {
                    Some(fields) => match fields.iter().position(|f| f == &field.name) {
                        Some(i) => i,
                        None => return false,
                    },
                    None => return false,
                };
                match self.types.type_meta.get(&struct_name) {
                    Some(meta) => meta.fields.get(idx).map(|(_, t)| Self::elem_name_is_int(t)).unwrap_or(false),
                    None => false,
                }
            }
            Expr::Call(callee, ..) | Expr::GenericCall(callee, ..) => {
                // BUG 22 #11 fix: resolve the callee's REGISTERED return type
                // (bare leaf, receiver-qualified "Vec.len", or module-qualified
                // keys -- the previous key construction used the receiver
                // EXPRESSION name ("v.len"), which never matched, so inline
                // method-call operands like `"len = " + v.len()` fell through
                // to inttoptr (garbage pointer -> AV).
                if let Some(rt) = self.callee_return_xiom(callee) {
                    return Self::elem_name_is_int(&rt);
                }
                // Inline Vec builtins have no registered signature: len -> Int.
                if let Expr::Field(obj, m, _) = callee.as_ref() {
                    let is_vec_recv = self.infer_struct_type_name(obj)
                        .map(|n| n == "Vec" || n.ends_with(".Vec"))
                        .unwrap_or(false)
                        || matches!(obj.as_ref(), Expr::Ident(id) if self.local.local_vec_elem.contains_key(&id.name));
                    if is_vec_recv && m.name == "len" {
                        return true;
                    }
                }
                let key = match callee.as_ref() {
                    Expr::Ident(id) => id.name.clone(),
                    Expr::Field(obj, m, _) => {
                        if let Expr::Ident(o) = obj.as_ref() {
                            format!("{}.{}", o.name, m.name)
                        } else {
                            return false;
                        }
                    }
                    _ => return false,
                };
                let ret_llvm = self.types.functions.get(&key)
                    .map(|(_, r)| r.clone())
                    .unwrap_or_default();
                ret_llvm.starts_with('i') && !ret_llvm.ends_with('*') && ret_llvm != "i8*"
            }
            _ => false,
        }
    }

    /// round-7 (ve2 follow-up): XIOM-level verdict -- is this expression a
    /// POINTER (raw `*T`, `&T`, `&mut T`), not a Str? The Str-concat intercept
    /// fires on ANY i8* operand at the LLVM level, but a `*UInt8` byte buffer
    /// (e.g. the Vec `data` field) is also i8* -- `buf + i` must be byte
    /// pointer arithmetic (GEP), never xiom_str_concat. Idents resolve through
    /// the registered XIOM type (local_xiom_types wins over the LLVM reverse
    /// lookup, which maps every i8* to "Str"); fields resolve through type_meta.
    pub(crate) fn expr_is_pointer(&self, e: &Expr) -> bool {
        match e {
            Expr::Ident(id) => {
                if let Some(t) = self.local.local_xiom_types.get(&id.name) {
                    return t.starts_with('*') || t.starts_with('&');
                }
                // Fallback: raw pointer slot (i8*/i64*/%struct.X*) -- but a
                // plain Str local also slots as i8*. Only report pointer when
                // the slot type is NOT i8* (i8* stays "possibly Str" so the
                // concat path keeps working for untyped Str values).
                if let Some((_, llvm_ty)) = self.lookup_local(&id.name) {
                    if llvm_ty.ends_with('*') && llvm_ty != "i8*" {
                        return true;
                    }
                }
                false
            }
            Expr::Field(obj, field, _) => {
                let struct_name = match self.infer_struct_type_name(obj) {
                    Some(n) => n,
                    None => return false,
                };
                let idx = match self.types.types.get(&struct_name) {
                    Some(fields) => match fields.iter().position(|f| f == &field.name) {
                        Some(i) => i,
                        None => return false,
                    },
                    None => return false,
                };
                match self.types.type_meta.get(&struct_name) {
                    Some(meta) => meta.fields.get(idx)
                        .map(|(_, t)| t.starts_with('*') || t.starts_with('&'))
                        .unwrap_or(false),
                    None => false,
                }
            }
            Expr::Paren(inner, _) => self.expr_is_pointer(inner),
            Expr::As(_, ty, _) => {
                let n = Self::type_from_ast(ty);
                n.starts_with('*') || n.starts_with('&')
            }
            _ => false,
        }
    }

    /// Convert an i64 (recovered from the trampoline's TLS result slot on the
    /// SUCCESS path of a confined unsafe block) back to the block's actual tail
    /// LLVM type. Inverse of val_to_i64. Used by the Expr::Unsafe arm after a
    /// xiom_trampoline_call returns 0 (no fault).
    pub(crate) fn unsafe_result_i64_to_val(&mut self, res: &str, last_ty: &str) -> (String, String) {
        if last_ty == "void" || last_ty.is_empty() {
            return (res.to_string(), LLVM_I64.to_string());
        }
        if last_ty == "double" {
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {bc} = bitcast i64 {res} to double"));
            return (bc, "double".to_string());
        }
        if last_ty == "float" {
            let tr = self.fresh_tmp();
            self.emitln(&format!("  {tr} = trunc i64 {res} to i32"));
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {bc} = bitcast i32 {tr} to float"));
            return (bc, "float".to_string());
        }
        if last_ty == "i8" || last_ty == "i16" || last_ty == "i32" {
            let tr = self.fresh_tmp();
            self.emitln(&format!("  {tr} = trunc i64 {res} to {last_ty}"));
            return (tr, last_ty.to_string());
        }
        if last_ty == "i1" {
            let tr = self.fresh_tmp();
            self.emitln(&format!("  {tr} = trunc i64 {res} to i1"));
            return (tr, "i1".to_string());
        }
        if last_ty.contains('*') {
            // Pointer tail (e.g. Str as i8*): re-materialize via inttoptr.
            let itp = self.fresh_tmp();
            self.emitln(&format!("  {itp} = inttoptr i64 {res} to {last_ty}"));
            return (itp, last_ty.to_string());
        }
        if last_ty.starts_with('%') {
            // Struct-typed tail: val_to_i64 stored a heap pointer to the boxed
            // struct. Re-materialize by loading through it.
            let itp = self.fresh_tmp();
            self.emitln(&format!("  {itp} = inttoptr i64 {res} to {last_ty}*"));
            let loaded = self.fresh_tmp();
            self.emitln(&format!("  {loaded} = load {last_ty}, {last_ty}* {itp}, align 16"));
            return (loaded, last_ty.to_string());
        }
        // Default: i64 (Int, Bool, Char, pointers-as-i64).
        (res.to_string(), last_ty.to_string())
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
            // Array-buffer-to-Vec conversion.
            let len_slot = self.fresh_tmp();
            self.emitln(&format!("  {len_slot} = bitcast i8* {val} to i64*"));
            let len_val = self.fresh_tmp();
            self.emitln(&format!("  {len_val} = load i64, i64* {len_slot}"));
            let byte_count = self.fresh_tmp();
            self.emitln(&format!("  {byte_count} = mul i64 {len_val}, 8"));
            // 5c-E: malloc(0) may return NULL on Windows. Ensure at least 1 byte.
            let safe_count = self.fresh_tmp();
            self.emitln(&format!("  {safe_count} = or i64 {byte_count}, 1"));
            let heap_copy = self.emit_alloc(&safe_count);
            // 5c-E: malloc(0) may return NULL on some platforms (Windows).
            // Only trap on NULL when length > 0; zero-length Vecs use NULL.
            let is_empty = self.fresh_tmp();
            self.emitln(&format!("  {is_empty} = icmp eq i64 {len_val}, 0"));
            let is_null = self.fresh_tmp();
            self.emitln(&format!("  {is_null} = icmp eq i8* {heap_copy}, null"));
            // Trap: null AND NOT empty (i.e., allocation failed for non-zero size)
            let not_empty = self.fresh_tmp();
            self.emitln(&format!("  {not_empty} = xor i1 {is_empty}, true"));
            let trap_cond = self.fresh_tmp();
            self.emitln(&format!("  {trap_cond} = and i1 {is_null}, {not_empty}"));
            let ok = self.fresh_block("arr_to_vec_ok");
            let fail = self.fresh_block("arr_to_vec_fail");
            self.emitln(&format!("  br i1 {trap_cond}, label %{fail}, label %{ok}"));
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
            // Ensure minimum capacity of 4 for empty arrays (push needs cap > len)
            let cap_needs_min = self.fresh_tmp();
            self.emitln(&format!("  {cap_needs_min} = icmp slt i64 {len_val}, 4"));
            let cap_val = self.fresh_tmp();
            self.emitln(&format!("  {cap_val} = select i1 {cap_needs_min}, i64 4, i64 {len_val}"));
            self.emitln(&format!("  store i64 {cap_val}, i64* {g2}"));
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
            let num_fields = self.types.types.get(&type_name.to_string())
                .or_else(|| {
                    let suffix = format!(".{}", type_name);
                    self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix))
                        .and_then(|k|self.types.types.get(&k))
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
