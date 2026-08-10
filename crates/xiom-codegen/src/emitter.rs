use super::IrEmitter;
use crate::llvm_consts::*;
use xiom_ast::*;
use std::collections::{HashMap, HashSet};

impl IrEmitter {
    pub(crate) fn fresh_tmp(&mut self) -> String {
        let n = self.tmp_counter;
        self.tmp_counter += 1;
        format!("%tmp{n}")
    }

    pub(crate) fn fresh_block(&mut self, label: &str) -> String {
        let n = self.block_counter;
        self.block_counter += 1;
        format!("{label}{n}")
    }

    pub(crate) fn push_scope(&mut self) {
        self.fctx.locals.push(HashMap::new());
    }

    pub(crate) fn pop_scope(&mut self) {
        self.fctx.locals.pop();
    }

    pub(crate) fn add_local(&mut self, name: &str, reg: String, llvm_ty: &str) {
        if let Some(scope) = self.fctx.locals.last_mut() {
            scope.insert(name.to_string(), (reg, llvm_ty.to_string()));
        }
    }

    pub(crate) fn lookup_local(&self, name: &str) -> Option<&(String, String)> {
        for scope in self.fctx.locals.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    /// M17: Returns `true` if the local variable `name` has a signed integer type
    /// (Int, Int8, Int16, Int32, Int64). Used to choose sext vs zext when widening.
    pub(crate) fn is_signed_local(&self, name: &str) -> bool {
        self.local.signed_locals.contains(name)
    }

    /// M17: Returns the XIOM type name for a local variable, if tracked.
    /// Prefers the explicit `local_xiom_types` map; falls back to LLVM-type reverse lookup.
    #[allow(dead_code)]
    pub(crate) fn xiom_type_of_local(&self, name: &str) -> Option<String> {
        if let Some(xiom_ty) = self.local.local_xiom_types.get(name) {
            return Some(xiom_ty.clone());
        }
        self.resolve_local_xiom_type(name)
    }

    /// Resolve a local variable's XIOM type name, with array-element awareness.
    /// For array-literal locals (in `array_locals`), returns the ELEMENT type
    /// (e.g. "Int" for `[5]Int`) instead of the buffer pointer type ("Str").
    pub(crate) fn resolve_local_xiom_type(&self, name: &str) -> Option<String> {
        if let Some((_, llvm_ty)) = self.lookup_local(name) {
            if self.local.array_locals.contains(name) {
                if let Some(elem_llvm) = self.local.local_array_elem.get(name) {
                    return Some(Self::xiom_type_name_from_llvm(elem_llvm));
                }
                return Some("Int".to_string());
            }
            Some(Self::xiom_type_name_from_llvm(llvm_ty))
        } else {
            None
        }
    }

    pub(crate) fn emitln(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }

    /// M20-A1: Emit any deferred closure function definitions.
    /// Called at the end of compile_fn so closures appear as top-level
    /// LLVM function definitions after the enclosing function.
    pub(crate) fn flush_deferred_closures(&mut self) {
        for def in std::mem::take(&mut self.local.deferred_closure_defs) {
            self.output.push_str(&def);
        }
    }

    /// M20: Compile an expression as an lvalue (pointer to its storage).
    /// Returns Some((pointer_reg, pointer_llvm_ty, element_llvm_ty)) or None.
    /// Supports: Ident (local variable), Field (struct field chain).
    pub(crate) fn compile_lvalue(&mut self, expr: &Expr) -> Option<(String, String, String)> {
        match expr {
            Expr::Ident(id) => {
                if let Some((alloca, llvm_ty)) = self.lookup_local(&id.name).cloned() {
                    // For pointer-typed locals (&mut T, &T, *T), load the pointer
                    // value from the alloca so subsequent GEPs operate on the actual
                    // pointee address rather than the alloca slot itself.
                    let (base_ptr, base_ptr_ty) = if llvm_ty.ends_with('*') {
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load {llvm_ty}, {llvm_ty}* {alloca}"));
                        (loaded, llvm_ty)
                    } else {
                        (alloca, format!("{llvm_ty}*"))
                    };
                    let elem_ty = base_ptr_ty.trim_end_matches('*').to_string();
                    Some((base_ptr, base_ptr_ty, elem_ty))
                } else { None }
            }
            Expr::Field(obj, field, _) => {
                // Get the object's type and field info
                let (obj_ptr, _obj_ptr_ty, obj_elem_ty) = self.compile_lvalue(obj)?;
                // Load the struct value to get its type
                let struct_ty = if obj_elem_ty.starts_with("%struct.") {
                    obj_elem_ty.clone()
                } else {
                    return None;
                };
                // Find the field index
                let type_name = struct_ty.trim_start_matches("%struct.");
                let field_idx = self.types.types.get(&type_name.to_string())
                    .and_then(|fields| fields.iter().position(|f| f == &field.name))?;
                let field_llvm_ty = self.field_llvm_type(type_name, field_idx);
                let gep = self.fresh_tmp();
                self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {obj_ptr}, i32 0, i32 {field_idx}"));
                let ptr_ty = format!("{field_llvm_ty}*");
                Some((gep, ptr_ty, field_llvm_ty))
            }
            // 5c.38: `container[i]` — compute the element address within a
            // Vec/Slice data buffer so field mutations (e.g. `items[0].x = v`)
            // can store through the resulting lvalue pointer.
            Expr::Index(container, index, _) => {
                let (idx_val, idx_ty) = self.compile_expr(index).ok()?;
                let idx_i64 = self.val_to_i64(&idx_val, &idx_ty);
                let vty = "%struct.Vec";
                // M33: For compound containers (field access through &mut),
                // resolve the container to an lvalue first so the Vec data
                // pointer is loaded from the ORIGINAL struct field, not a
                // temp copy. store_back_to_receiver then writes through
                // the correct element pointer.
                let (dp_val, esz_val) = if let Some((lv_ptr, lv_ptr_ty, _)) = self.compile_lvalue(container) {
                    if lv_ptr_ty == "%struct.Vec*" || lv_ptr_ty.ends_with(".Vec*") || lv_ptr_ty == "%struct.Slice*" || lv_ptr_ty.ends_with(".Slice*") {
                        // Load data ptr and elem size from the original Vec field
                        let dp = self.fresh_tmp();
                        let esz = self.fresh_tmp();
                        let dp_gep = self.fresh_tmp();
                        let esz_gep = self.fresh_tmp();
                        self.emitln(&format!("  {dp_gep} = getelementptr %struct.Vec, {lv_ptr_ty} {lv_ptr}, i32 0, i32 0"));
                        self.emitln(&format!("  {dp} = load i8*, i8** {dp_gep}"));
                        self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, {lv_ptr_ty} {lv_ptr}, i32 0, i32 3"));
                        self.emitln(&format!("  {esz} = load i64, i64* {esz_gep}"));
                        (dp, esz)
                    } else {
                        return None;
                    }
                } else {
                    // Fallback: Ident or temp copy
                    let vslot = if let Expr::Ident(cont_id) = container.as_ref() {
                        if let Some((slot, _)) = self.lookup_local(&cont_id.name).cloned() {
                            slot
                        } else {
                            return None;
                        }
                    } else {
                        let (cont_val, _) = self.compile_expr(container).ok()?;
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = alloca {vty}"));
                        self.emitln(&format!("  {tmp}_i8 = bitcast {vty}* {tmp} to i8*"));
                        self.emitln(&format!("  call void @llvm.memset.p0i8.i64(i8* {tmp}_i8, i8 0, i64 32, i1 false)"));
                        self.emit_vec_store_fields(&cont_val, &tmp);
                        tmp
                    };
                    let dp_gep = self.fresh_tmp();
                    let dp_val = self.fresh_tmp();
                    self.emitln(&format!("  {dp_gep} = getelementptr {vty}, {vty}* {vslot}, i32 0, i32 0"));
                    self.emitln(&format!("  {dp_val} = load i8*, i8** {dp_gep}"));
                    let esz_gep = self.fresh_tmp();
                    let esz_val = self.fresh_tmp();
                    self.emitln(&format!("  {esz_gep} = getelementptr {vty}, {vty}* {vslot}, i32 0, i32 3"));
                    self.emitln(&format!("  {esz_val} = load i64, i64* {esz_gep}"));
                    (dp_val, esz_val)
                };
                // Compute element byte offset
                let byte_off = self.fresh_tmp();
                self.emitln(&format!("  {byte_off} = mul i64 {idx_i64}, {esz_val}"));
                let elem_ptr = self.fresh_tmp();
                self.emitln(&format!("  {elem_ptr} = getelementptr i8, i8* {dp_val}, i64 {byte_off}"));
                // Infer element LLVM type. For struct element access,
                // bitcast the i8* element pointer to the correct struct pointer
                // so the caller can GEP into the struct's fields.
                let elem_ty = self.infer_vec_elem_llvm_type(container);
                let typed_ptr = if elem_ty.starts_with("%struct.") || elem_ty.ends_with('*') {
                    let cast = self.fresh_tmp();
                    self.emitln(&format!("  {cast} = bitcast i8* {elem_ptr} to {elem_ty}*"));
                    cast
                } else {
                    elem_ptr
                };
                Some((typed_ptr, format!("{elem_ty}*"), elem_ty))
            }
            _ => None,
        }
    }

    /// M20-A1: Collect free (captured) variables from a closure body expression.
    /// Returns Vec of (var_name, llvm_type) for each local variable referenced
    /// in the body that is NOT in the param list.
    pub(crate) fn collect_free_vars(&self, expr: &Expr, param_names: &[String]) -> Vec<(String, String)> {
        let mut used = std::collections::HashSet::new();
        self.collect_ident_names(expr, &mut used);
        let mut captures = Vec::new();
        for name in used {
            if !param_names.contains(&name) {
                if let Some((_, llvm_ty)) = self.lookup_local(&name) {
                    captures.push((name.clone(), llvm_ty.clone()));
                }
            }
        }
        captures
    }

    /// M20-A1: Collect free variables from a block body (for fn-style closures).
    pub(crate) fn collect_block_free_vars(&self, block: &Block, param_names: &[String]) -> Vec<(String, String)> {
        let mut used = std::collections::HashSet::new();
        for stmt in &block.stmts {
            match stmt {
                StmtOrExpr::Expr(e) => self.collect_ident_names(e, &mut used),
                StmtOrExpr::Stmt(s) => self.collect_stmt_names_inner(s, &mut used),
            }
        }
        let mut captures = Vec::new();
        for name in used {
            if !param_names.contains(&name) {
                if let Some((_, llvm_ty)) = self.lookup_local(&name) {
                    captures.push((name.clone(), llvm_ty.clone()));
                }
            }
        }
        captures
    }

    /// Recursively collect all identifier names from an expression.
    fn collect_ident_names(&self, expr: &Expr, out: &mut std::collections::HashSet<String>) {
        match expr {
            Expr::Ident(id) => { out.insert(id.name.clone()); }
            Expr::Binary(left, _, right, _) => {
                self.collect_ident_names(left, out);
                self.collect_ident_names(right, out);
            }
            Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => {
                self.collect_ident_names(func, out);
                for a in args { self.collect_ident_names(a, out); }
            }
            Expr::Field(obj, _, _) | Expr::Index(obj, _, _) | Expr::Ref(obj, _) => {
                self.collect_ident_names(obj, out);
            }
            Expr::Unary(_, obj, _) => {
                self.collect_ident_names(obj, out);
            }
            Expr::If(cond, then_block, _elifs, _else_block, _) => {
                self.collect_ident_names(cond, out);
                for stmt in &then_block.stmts {
                    self.collect_stmt_names(stmt, out);
                }
            }
            Expr::PipeClosure(_, inner, _) => {
                self.collect_ident_names(inner, out);
            }
            _ => {}
        }
    }

    /// Collect identifier names from a statement-or-expression.
    fn collect_stmt_names(&self, stmt: &StmtOrExpr, out: &mut std::collections::HashSet<String>) {
        match stmt {
            StmtOrExpr::Stmt(s) => self.collect_stmt_names_inner(s, out),
            StmtOrExpr::Expr(e) => self.collect_ident_names(e, out),
        }
    }
    
    fn collect_stmt_names_inner(&self, stmt: &Stmt, out: &mut std::collections::HashSet<String>) {
        match stmt {
            Stmt::Expr(e, _) => self.collect_ident_names(e, out),
            Stmt::Let(_, _, e, _) => self.collect_ident_names(e, out),
            Stmt::Var(_, _, e, _) => self.collect_ident_names(e, out),
            Stmt::Return(Some(e), _) => self.collect_ident_names(e, out),
            Stmt::If(cond, _, _, _, _) => self.collect_ident_names(cond, out),
            Stmt::While(cond, _, _, _, _) => self.collect_ident_names(cond, out),
            _ => {}
        }
    }

    /// True if the most recently emitted line in the current function body is a
    /// basic-block terminator. Used to decide whether a fallback terminator must
    /// be appended so every block is terminated and the IR stays valid.
    ///
    /// Returns `false` when the last meaningful line is a bare label (a freshly
    /// opened, still-empty block) or a non-terminator instruction.
    pub(crate) fn current_block_terminated(&self) -> bool {
        for line in self.output.lines().rev() {
            let t = line.trim();
            if t.is_empty() || t.starts_with(';') {
                continue;
            }
            // A bare label line ("entry:", "endif7:") opens a fresh block that has
            // no terminator yet.
            if t.ends_with(':') && !t.contains(' ') {
                return false;
            }
            return t == "unreachable"
                || t == "ret void"
                || t.starts_with("ret ")
                || t.starts_with("br ")
                || t.starts_with("switch ");
        }
        false
    }

    /// Emit the builtin `declare` preamble: hardcoded C stdlib, LLVM intrinsics,
    /// and XIOM runtime symbols. Also pre-seeds `self.mono.already_declared` so user
    /// extern blocks never duplicate them.
    pub(crate) fn emit_builtin_declares(&mut self) {
        self.mono.already_declared = IrEmitter::hardcoded_declare_names();

        self.emitln("declare i32 @printf(i8*, ...)");
        self.emitln("declare i32 @sprintf(i8*, i8*, ...)");
        self.emitln("declare i32 @puts(i8*)");
        self.emitln("declare void @llvm.trap()");
        self.emitln("@xiom_recursion_counter = internal thread_local global i64 0");
        self.emitln("declare i8* @malloc(i64)");
        // D2.1 (Unsafe Confinement Phase 3): guard-heap arena + Copy-Out.
        self.emitln("declare void @xiom_guard_heap_enter()");
        self.emitln("declare void @xiom_guard_heap_exit()");
        self.emitln("declare i8* @xiom_guard_alloc(i64)");
        self.emitln("declare i8* @xiom_guard_copy_out(i8*, i64)");
        self.emitln("declare i8* @xiom_guard_copy_str(i8*)");
        // D2.1 (Phase 4): stack guard pages (red-zone overflow catch).
        self.emitln("declare void @xiom_guard_page_arm()");
        self.emitln("declare void @xiom_guard_page_disarm()");
        self.emitln("declare i8* @realloc(i8*, i64)");
        self.emitln("declare void @free(i8*)");
        self.emitln("declare void @llvm.memcpy.p0i8.p0i8.i64(i8*, i8*, i64, i1)");
        self.emitln("declare void @llvm.memmove.p0i8.p0i8.i64(i8*, i8*, i64, i1)");
        self.emitln("declare i64 @xiom_is_sorted(i8*)");
        self.emitln("declare i64 @xiom_all(i8*, i64, i8*)");
        self.emitln("declare i64 @xiom_none(i8*, i64, i8*)");
        self.emitln("declare i64 @xiom_contains(i8*, i64)");
        self.emitln("declare i8* @xiom_read_file(i8*)");
        self.emitln("declare i64 @xiom_file_size(i8*)");
        self.emitln("declare void @xiom_free(i8*)");
        self.emitln("declare i8 @xiom_char_at(i8*, i64)");
        self.emitln("declare i64 @xiom_str_len(i8*)");
        // Always declare strcmp — used for Str == / != content comparison.
        // (Identical duplicate declares are legal in LLVM; the metadata-table
        // path may also emit it, which is harmless.)
        self.emitln("declare i32 @strcmp(i8*, i8*)");
        // Runtime string concatenation — used for Str + Str lowering.
        self.emitln("declare i8* @xiom_str_concat(i8*, i8*)");
        // D1 hardening: NUL-terminating copy for Str::from_utf8(Vec[UInt8]).
        self.emitln("declare i8* @xiom_str_from_vec(i8*, i64)");
        // M12/P1: Runtime string slice/starts_with/ends_with — scripting ergonomics.
        self.emitln("declare i8* @xiom_str_slice(i8*, i64, i64)");
        self.emitln("declare i1 @xiom_str_starts_with(i8*, i8*)");
        self.emitln("declare i1 @xiom_str_ends_with(i8*, i8*)");
        // Runtime integer→string — used for to_string(Int) / Int.to_str().
        self.emitln("declare i8* @xiom_int_to_string(i64)");
        // String interning
        self.emitln("declare i64 @xiom_intern(i8*, i64, i64)");
        self.emitln("declare i8* @xiom_lookup(i64)");
        // IR emission
        self.emitln("declare i64 @xiom_ir_open(i8*)");
        self.emitln("declare void @xiom_ir_close()");
        self.emitln("declare void @xiom_ir_header()");
        self.emitln("declare void @xiom_ir_define(i64, i64)");
        self.emitln("declare void @xiom_ir_param(i64, i64)");
        self.emitln("declare void @xiom_ir_entry()");
        self.emitln("declare void @xiom_ir_alloca(i64, i64)");
        self.emitln("declare void @xiom_ir_store(i64, i64, i64)");
        self.emitln("declare void @xiom_ir_load(i64, i64, i64)");
        self.emitln("declare void @xiom_ir_binop(i8*, i64, i64, i64, i64)");
        self.emitln("declare void @xiom_ir_call(i64, i64, i64)");
        self.emitln("declare void @xiom_ir_call_arg(i64, i64)");
        self.emitln("declare void @xiom_ir_call_lit(i8*)");
        self.emitln("declare void @xiom_ir_call_end()");
        self.emitln("declare void @xiom_ir_ret(i64, i64)");
        self.emitln("declare void @xiom_ir_ret_void()");
        self.emitln("declare void @xiom_ir_endfn()");
        self.emitln("declare void @xiom_ir_raw(i8*)");
        self.emitln("declare void @xiom_ir_emit_program(i64)");
        // v0.9.4 string-based IR emission
        self.emitln("declare void @xiom_ir_define_s(i8*, i8*)");
        self.emitln("declare void @xiom_ir_param_int(i64)");
        self.emitln("declare void @xiom_ir_param_double(i64)");
        self.emitln("declare void @xiom_ir_alloca_s(i64)");
        self.emitln("declare void @xiom_ir_store_param(i64, i64)");
        self.emitln("declare void @xiom_ir_load_s(i64, i64)");
        self.emitln("declare void @xiom_ir_add(i64, i64, i64)");
        self.emitln("declare void @xiom_ir_fmul(i64, i64, i64)");
        self.emitln("declare void @xiom_ir_call_fn(i64, i8*, i8*)");
        self.emitln("declare void @xiom_ir_call_arg_lit(i8*, i8*)");
        self.emitln("declare void @xiom_ir_ret_reg(i64)");
        self.emitln("declare void @xiom_ir_ret_lit(i64)");
        // v0.10.0 function table (IR self-hosting)
        self.emitln("declare void @xiom_fn_table_init()");
        self.emitln("declare void @xiom_set_source(i64)");
        self.emitln("declare void @xiom_fn_table_add(i64, i64, i64, i64, i64)");
        self.emitln("declare i64 @xiom_fn_table_count()");
        self.emitln("declare i64 @xiom_fn_name_id(i64)");
        self.emitln("declare i64 @xiom_fn_ret_type_id(i64)");
        self.emitln("declare i64 @xiom_fn_param_count(i64)");
        self.emitln("declare i64 @xiom_fn_body_start(i64)");
        self.emitln("declare i64 @xiom_fn_body_end(i64)");
        self.emitln("declare i64 @xiom_fn_emit_all()");
        // 5e.5a: hot reload function pointer table (only when enabled)
        if self.config.hot_reload {
            self.emitln("declare i64 @xiom_hot_get_ptr(i64)");
            self.emitln("declare void @xiom_hot_set_ptr(i64, i64)");
            // 5e.5c: state migration — file I/O for global save/restore
            self.emitln("declare i8* @fopen(i8*, i8*)");
            self.emitln("declare i64 @fwrite(i8*, i64, i64, i8*)");
            self.emitln("declare i64 @fread(i8*, i64, i64, i8*)");
            self.emitln("declare i32 @fclose(i8*)");
            // 7D.2: string comparison for layout metadata verification
            self.emitln("declare i32 @strncmp(i8*, i8*, i64)");
        }
        self.emitln("");
    }

    /// R1: Emit DWARF debug info metadata for .xi source-level debugging.
    /// Emits !llvm.dbg.cu, !llvm.module.flags, !DIFile, and !DICompileUnit
    /// at the LLVM IR module level. Functions then attach !dbg !{subprogram}
    /// to their define lines for source-level breakpoints in GDB/LLDB.
    pub(crate) fn emit_debug_metadata(&mut self) {
        if !self.config.debug_symbols {
            return;
        }
        // Module flags for DWARF version and debug info version
        self.emitln("!llvm.dbg.cu = !{!0}");
        self.emitln("!llvm.module.flags = !{!1, !2, !3}");
        self.emitln("!1 = !{i32 2, !\"Dwarf Version\", i32 4}");
        self.emitln("!2 = !{i32 2, !\"Debug Info Version\", i32 3}");
        self.emitln("!3 = !{i32 1, !\"wchar_size\", i32 2}");

        // DIFile: source file name and directory
        let source = &self.config.source_file;
        let dir = std::path::Path::new(source)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        let filename = std::path::Path::new(source)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown.xi".to_string());
        self.emitln(&format!("!4 = !DIFile(filename: \"{}\", directory: \"{}\")", filename, dir));

        // DICompileUnit: language = DW_LANG_C99 (0x000c), producer = "XIOM"
        self.emitln("!0 = distinct !DICompileUnit(language: DW_LANG_C99, file: !4, producer: \"XIOM v0.56\", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug)");
        self.emitln("");

        // Track DI node counter for function subprograms
        self.local.di_node_counter = 5; // 0-4 used above
    }

    /// Emit `define` stubs for any `@symbol` that is *called* in the emitted IR
    /// but never `define`d or `declare`d. LLVM/clang rejects such references, but
    /// they legitimately occur in erased-generic dead code (method bodies that
    /// were monomorphised away leave behind unresolved bare method calls). Each
    /// stub returns a typed default matching the return type observed at a call
    /// site. clang tolerates call/definition signature mismatches, so a single
    /// zero-arg stub satisfies every call form for that symbol.
    pub(crate) fn emit_undefined_symbol_stubs(&mut self) {
        use std::collections::{HashMap, HashSet};
        let mut defined: HashSet<String> = HashSet::new();
        let mut declared: HashSet<String> = HashSet::new();
        // Preferred return type per called symbol (first non-void wins).
        let mut called: HashMap<String, String> = HashMap::new();

        let take_name = |rest: &str| -> Option<String> {
            // rest begins right after '@'; take the identifier up to '('.
            let mut end = 0;
            for (i, c) in rest.char_indices() {
                if c == '(' { end = i; break; }
                if !(c.is_ascii_alphanumeric() || c == '_' || c == '.') { return None; }
            }
            if end == 0 { return None; }
            Some(rest[..end].to_string())
        };

        for line in self.output.lines() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix("define ") {
                if let Some(at) = rest.find('@') {
                    if let Some(name) = take_name(&rest[at + 1..]) {
                        defined.insert(name);
                    }
                }
            } else if let Some(rest) = t.strip_prefix("declare ") {
                if let Some(at) = rest.find('@') {
                    if let Some(name) = take_name(&rest[at + 1..]) {
                        declared.insert(name);
                    }
                }
            }
            // Match `... call <rettype> @name(` — capture the token before '@'.
            if let Some(cpos) = t.find("call ") {
                let after = &t[cpos + 5..];
                if let Some(at) = after.find('@') {
                    // The return type is the single token immediately before '@'.
                    let head = after[..at].trim_end();
                    // Skip complex call forms (e.g. `i64 (i8*, ...) @printf`) whose
                    // token-before-@ is a ')'; those callees are always declared.
                    if let Some(ret_ty) = head.rsplit(char::is_whitespace).next() {
                        if !ret_ty.is_empty() && !ret_ty.ends_with(')') {
                            if let Some(name) = take_name(&after[at + 1..]) {
                                let entry = called.entry(name).or_insert_with(|| ret_ty.to_string());
                                if *entry == "void" && ret_ty != "void" {
                                    *entry = ret_ty.to_string();
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut missing: Vec<(String, String)> = called
            .into_iter()
            .filter(|(name, _)| {
                !defined.contains(name)
                    && !declared.contains(name)
                    && !name.starts_with("llvm.")
                    // v0.55/v0.56: Skip C runtime functions provided by external linkage
                    && name != "xiom_thread_spawn"
                    && name != "xiom_channel_create"
                    && name != "xiom_channel_send"
                    && name != "xiom_channel_recv"
                    && name != "xiom_channel_try_recv"
                    && name != "xiom_threadpool_init"
                    && name != "xiom_threadpool_spawn"
                    // D1 hardening: provided by xiom_runtime.c (Str::from_utf8
                    // NUL-termination helper). Must not be auto-stubbed — the
                    // runtime defines it, so a stub would duplicate the symbol.
                    && name != "xiom_str_from_vec"
            })
            .collect();
        if missing.is_empty() {
            return;
        }
        missing.sort();
        self.emitln("");
        self.emitln("; --- auto-stubs for erased-generic dead-code callees ---");
        for (name, ret_ty) in missing {
            if ret_ty == "void" {
                self.emitln(&format!("define void @{name}() {{"));
                self.emitln("entry:");
                self.emitln("  ret void");
                self.emitln("}");
            } else {
                let default = Self::default_const_for(&ret_ty);
                self.emitln(&format!("define {ret_ty} @{name}() {{"));
                self.emitln("entry:");
                self.emitln(&format!("  ret {ret_ty} {default}"));
                self.emitln("}");
            }
        }
    }

    /// Walk all top-level declarations (recursing into modules) and emit
    /// `declare` statements for every `extern "C"` function whose name is not
    /// already present in `self.mono.already_declared`. Skips functions whose names
    /// are already in the hardcoded set or already declared by another extern block.
    pub(crate) fn emit_extern_declares(&mut self, items: &[TopDecl]) {
        for item in items {
            match item {
                TopDecl::Extern(eb) => {
                    for fd in &eb.functions {
                        let name = &fd.name.name;
                        // Track thread spawn declaration to avoid duplicate builtin
                        if name == "xiom_thread_spawn" {
                            self.local.spawn_declared = true;
                        }
                        if self.mono.already_declared.contains(name) {
                            continue;
                        }
                        self.mono.already_declared.insert(name.clone());
                        // Map return type
                        let ret_llvm = fd.return_type.as_ref()
                            .map(|t| self.extern_type_to_llvm(t))
                            .unwrap_or_else(|| "void".to_string());
                        // Map param types
                        let param_llvm: Vec<String> = fd.params.iter()
                            .map(|p| self.extern_type_to_llvm(&p.ty))
                            .collect();
                        // NOTE: Variadic extern functions (with `...` in the source)
                        // are parsed but the variadic marker is not stored in FnDecl.
                        // Therefore we cannot detect variadics from the AST alone.
                        // All extern declares are emitted without `...`.
                        // If you add variadic detection, change the last arg to `...`.
                        let params_str = if param_llvm.is_empty() {
                            "".to_string()
                        } else {
                            param_llvm.join(", ")
                        };
                        self.emitln(&format!("declare {ret_llvm} @{name}({params_str})"));
                    }
                }
                TopDecl::Module(md) => {
                    self.emit_extern_declares(&md.items);
                }
                _ => {}
            }
        }
    }

    // ========================================================================
    // Additive metadata tables (RTTI + contracts)
    //
    // Everything below is a NEW emission surface for the `reflect` and
    // `contracts` stdlib modules. It is strictly ADDITIVE:
    //   * it only READS already-registered state (`type_meta`, `enum_variants`)
    //     and the program AST,
    //   * it only WRITES new globals, new `@xiom_*` function definitions, and
    //     new entries into `self.types.functions` (never overwriting existing keys),
    //   * it is gated so it emits nothing unless the program actually declares
    //     the corresponding `extern "C"` accessors (only reflect.xi /
    //     contracts.xi do), keeping all other programs identical.
    // ========================================================================

    /// Emit the additive RTTI + contract metadata tables and their fixed-ABI
    /// accessor functions. See the section header above for the additivity
    /// guarantees. Does nothing unless the program declares the accessors.
    pub(crate) fn emit_metadata_tables(&mut self, program: &Program) {
        let want_reflect = IrEmitter::program_declares_extern(&program.items, "xiom_type_count");
        let want_contracts = IrEmitter::program_declares_extern(&program.items, "xiom_contract_fn_count");
        if !want_reflect && !want_contracts {
            return;
        }
        self.emitln("; ---- XIOM additive metadata (RTTI / contracts) ----");
        // NOTE: strcmp is already declared unconditionally in the main declare
        // block (used for Str == / != content comparison), so we must NOT declare
        // it again here — LLVM rejects duplicate function declarations.
        if want_reflect {
            self.emit_rtti_table();
        }
        if want_contracts {
            self.emit_contract_table(program);
        }
        self.emitln("");
    }

    /// PART A — read-only RTTI table + accessors for `reflect`.
    pub(crate) fn emit_rtti_table(&mut self) {
        // Collect user types in a deterministic (sorted) order, excluding the
        // compiler's builtin/synthetic types. The type id is the index here.
        let mut names: Vec<String> = self
            .types.type_meta
            .keys().into_iter()
     .filter(|n| {
                let n = n.as_str();
                n != "Option" && n != "Result" && n != "Vec" && n != "Tuple" && !n.starts_with("Tuple_")
            })
            .collect();
        names.sort();
        let n = names.len();
        // Field counts: 0 for enums, otherwise the number of fields.
        let field_counts: Vec<usize> = names
            .iter()
            .map(|name| {
                if self.types.enum_variants.contains_key(name) {
                    0
                } else {
                    self.types.type_meta.get(name).map(|m| m.fields.len()).unwrap_or(0)
                }
            })
            .collect();

        // Per-type name string constants.
        for (i, name) in names.iter().enumerate() {
            let escaped = IrEmitter::escape_ir_string(name);
            self.emitln(&format!(
                "@.xiom_rtti_name_{i} = private unnamed_addr constant [{len} x i8] c\"{escaped}\\00\"",
                len = name.len() + 1
            ));
        }
        // Fallback name for out-of-range ids.
        self.emitln("@.xiom_rtti_unknown = private unnamed_addr constant [8 x i8] c\"unknown\\00\"");

        // Parallel arrays of name pointers and field counts.
        if n == 0 {
            self.emitln("@.xiom_rtti_names = private unnamed_addr constant [0 x i8*] zeroinitializer");
            self.emitln("@.xiom_rtti_field_counts = private unnamed_addr constant [0 x i64] zeroinitializer");
        } else {
            let name_elems: Vec<String> = names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    format!(
                        "i8* getelementptr inbounds ([{len} x i8], [{len} x i8]* @.xiom_rtti_name_{i}, i64 0, i64 0)",
                        len = name.len() + 1
                    )
                })
                .collect();
            self.emitln(&format!(
                "@.xiom_rtti_names = private unnamed_addr constant [{n} x i8*] [{}]",
                name_elems.join(", ")
            ));
            let fc_elems: Vec<String> = field_counts.iter().map(|c| format!("i64 {c}")).collect();
            self.emitln(&format!(
                "@.xiom_rtti_field_counts = private unnamed_addr constant [{n} x i64] [{}]",
                fc_elems.join(", ")
            ));
        }

        // i64 @xiom_type_count()
        self.types.functions.insert("xiom_type_count".to_string(), (vec![], LLVM_I64.to_string()));
        self.emitln("define i64 @xiom_type_count() {");
        self.emitln("entry:");
        self.emitln(&format!("  ret i64 {n}"));
        self.emitln("}\n");

        // i8* @xiom_type_name(i64 %id) — name or "unknown" if out of range.
        self.types.functions
            .insert("xiom_type_name".to_string(), (vec![LLVM_I64.to_string()], LLVM_STR_PTR.to_string()));
        self.emitln("define i8* @xiom_type_name(i64 %id) {");
        self.emitln("entry:");
        self.emitln("  %lo = icmp slt i64 %id, 0");
        self.emitln(&format!("  %hi = icmp sge i64 %id, {n}"));
        self.emitln("  %oob = or i1 %lo, %hi");
        self.emitln("  br i1 %oob, label %oob_bb, label %ok_bb");
        self.emitln("oob_bb:");
        self.emitln("  %u = getelementptr [8 x i8], [8 x i8]* @.xiom_rtti_unknown, i64 0, i64 0");
        self.emitln("  ret i8* %u");
        self.emitln("ok_bb:");
        self.emitln(&format!(
            "  %p = getelementptr [{n} x i8*], [{n} x i8*]* @.xiom_rtti_names, i64 0, i64 %id"
        ));
        self.emitln("  %v = load i8*, i8** %p");
        self.emitln("  ret i8* %v");
        self.emitln("}\n");

        // i64 @xiom_type_field_count(i64 %id) — 0 if out of range.
        self.types.functions
            .insert("xiom_type_field_count".to_string(), (vec![LLVM_I64.to_string()], LLVM_I64.to_string()));
        self.emitln("define i64 @xiom_type_field_count(i64 %id) {");
        self.emitln("entry:");
        self.emitln("  %lo = icmp slt i64 %id, 0");
        self.emitln(&format!("  %hi = icmp sge i64 %id, {n}"));
        self.emitln("  %oob = or i1 %lo, %hi");
        self.emitln("  br i1 %oob, label %oob_bb, label %ok_bb");
        self.emitln("oob_bb:");
        self.emitln("  ret i64 0");
        self.emitln("ok_bb:");
        self.emitln(&format!(
            "  %p = getelementptr [{n} x i64], [{n} x i64]* @.xiom_rtti_field_counts, i64 0, i64 %id"
        ));
        self.emitln("  %v = load i64, i64* %p");
        self.emitln("  ret i64 %v");
        self.emitln("}\n");

        // i64 @xiom_type_id_by_name(i8* %name) — linear search, -1 if absent.
        self.types.functions
            .insert("xiom_type_id_by_name".to_string(), (vec![LLVM_STR_PTR.to_string()], LLVM_I64.to_string()));
        self.emitln("define i64 @xiom_type_id_by_name(i8* %name) {");
        self.emitln("entry:");
        self.emitln("  br label %loop");
        self.emitln("loop:");
        self.emitln("  %i = phi i64 [ 0, %entry ], [ %inext, %cont ]");
        self.emitln(&format!("  %done = icmp sge i64 %i, {n}"));
        self.emitln("  br i1 %done, label %notfound, label %body");
        self.emitln("body:");
        self.emitln(&format!(
            "  %np = getelementptr [{n} x i8*], [{n} x i8*]* @.xiom_rtti_names, i64 0, i64 %i"
        ));
        self.emitln("  %ns = load i8*, i8** %np");
        self.emitln("  %c = call i32 @strcmp(i8* %name, i8* %ns)");
        self.emitln("  %eq = icmp eq i32 %c, 0");
        self.emitln("  br i1 %eq, label %found, label %cont");
        self.emitln("cont:");
        self.emitln("  %inext = add i64 %i, 1");
        self.emitln("  br label %loop");
        self.emitln("found:");
        self.emitln("  ret i64 %i");
        self.emitln("notfound:");
        self.emitln("  ret i64 -1");
        self.emitln("}\n");
    }

    /// PART B — read-only contract metadata table + accessors for `contracts`.
    pub(crate) fn emit_contract_table(&mut self, program: &Program) {
        let mut entries: Vec<(String, usize, usize)> = Vec::new();
        IrEmitter::collect_contract_fns(&program.items, &mut entries);
        let m = entries.len();

        for (i, (name, _, _)) in entries.iter().enumerate() {
            let escaped = IrEmitter::escape_ir_string(name);
            self.emitln(&format!(
                "@.xiom_contract_name_{i} = private unnamed_addr constant [{len} x i8] c\"{escaped}\\00\"",
                len = name.len() + 1
            ));
        }
        self.emitln("@.xiom_contract_unknown = private unnamed_addr constant [8 x i8] c\"unknown\\00\"");

        if m == 0 {
            self.emitln("@.xiom_contract_names = private unnamed_addr constant [0 x i8*] zeroinitializer");
            self.emitln("@.xiom_contract_pre = private unnamed_addr constant [0 x i64] zeroinitializer");
            self.emitln("@.xiom_contract_post = private unnamed_addr constant [0 x i64] zeroinitializer");
        } else {
            let name_elems: Vec<String> = entries
                .iter()
                .enumerate()
                .map(|(i, (name, _, _))| {
                    format!(
                        "i8* getelementptr inbounds ([{len} x i8], [{len} x i8]* @.xiom_contract_name_{i}, i64 0, i64 0)",
                        len = name.len() + 1
                    )
                })
                .collect();
            self.emitln(&format!(
                "@.xiom_contract_names = private unnamed_addr constant [{m} x i8*] [{}]",
                name_elems.join(", ")
            ));
            let pre_elems: Vec<String> = entries.iter().map(|(_, pre, _)| format!("i64 {pre}")).collect();
            self.emitln(&format!(
                "@.xiom_contract_pre = private unnamed_addr constant [{m} x i64] [{}]",
                pre_elems.join(", ")
            ));
            let post_elems: Vec<String> = entries.iter().map(|(_, _, post)| format!("i64 {post}")).collect();
            self.emitln(&format!(
                "@.xiom_contract_post = private unnamed_addr constant [{m} x i64] [{}]",
                post_elems.join(", ")
            ));
        }

        // i64 @xiom_contract_fn_count()
        self.types.functions
            .insert("xiom_contract_fn_count".to_string(), (vec![], LLVM_I64.to_string()));
        self.emitln("define i64 @xiom_contract_fn_count() {");
        self.emitln("entry:");
        self.emitln(&format!("  ret i64 {m}"));
        self.emitln("}\n");

        // i8* @xiom_contract_fn_name(i64 %idx)
        self.types.functions
            .insert("xiom_contract_fn_name".to_string(), (vec![LLVM_I64.to_string()], LLVM_STR_PTR.to_string()));
        self.emitln("define i8* @xiom_contract_fn_name(i64 %idx) {");
        self.emitln("entry:");
        self.emitln("  %lo = icmp slt i64 %idx, 0");
        self.emitln(&format!("  %hi = icmp sge i64 %idx, {m}"));
        self.emitln("  %oob = or i1 %lo, %hi");
        self.emitln("  br i1 %oob, label %oob_bb, label %ok_bb");
        self.emitln("oob_bb:");
        self.emitln("  %u = getelementptr [8 x i8], [8 x i8]* @.xiom_contract_unknown, i64 0, i64 0");
        self.emitln("  ret i8* %u");
        self.emitln("ok_bb:");
        self.emitln(&format!(
            "  %p = getelementptr [{m} x i8*], [{m} x i8*]* @.xiom_contract_names, i64 0, i64 %idx"
        ));
        self.emitln("  %v = load i8*, i8** %p");
        self.emitln("  ret i8* %v");
        self.emitln("}\n");

        // i64 @xiom_contract_pre_count(i64 %idx)
        self.types.functions
            .insert("xiom_contract_pre_count".to_string(), (vec![LLVM_I64.to_string()], LLVM_I64.to_string()));
        self.emitln("define i64 @xiom_contract_pre_count(i64 %idx) {");
        self.emitln("entry:");
        self.emitln("  %lo = icmp slt i64 %idx, 0");
        self.emitln(&format!("  %hi = icmp sge i64 %idx, {m}"));
        self.emitln("  %oob = or i1 %lo, %hi");
        self.emitln("  br i1 %oob, label %oob_bb, label %ok_bb");
        self.emitln("oob_bb:");
        self.emitln("  ret i64 0");
        self.emitln("ok_bb:");
        self.emitln(&format!(
            "  %p = getelementptr [{m} x i64], [{m} x i64]* @.xiom_contract_pre, i64 0, i64 %idx"
        ));
        self.emitln("  %v = load i64, i64* %p");
        self.emitln("  ret i64 %v");
        self.emitln("}\n");

        // i64 @xiom_contract_post_count(i64 %idx)
        self.types.functions
            .insert("xiom_contract_post_count".to_string(), (vec![LLVM_I64.to_string()], LLVM_I64.to_string()));
        self.emitln("define i64 @xiom_contract_post_count(i64 %idx) {");
        self.emitln("entry:");
        self.emitln("  %lo = icmp slt i64 %idx, 0");
        self.emitln(&format!("  %hi = icmp sge i64 %idx, {m}"));
        self.emitln("  %oob = or i1 %lo, %hi");
        self.emitln("  br i1 %oob, label %oob_bb, label %ok_bb");
        self.emitln("oob_bb:");
        self.emitln("  ret i64 0");
        self.emitln("ok_bb:");
        self.emitln(&format!(
            "  %p = getelementptr [{m} x i64], [{m} x i64]* @.xiom_contract_post, i64 0, i64 %idx"
        ));
        self.emitln("  %v = load i64, i64* %p");
        self.emitln("  ret i64 %v");
        self.emitln("}\n");
    }
}

impl IrEmitter {
    /// Collect the set of all hardcoded `declare` names so user extern blocks
    /// never duplicate them. This set is pre-seeded before emitting user extern
    /// function declares.
    pub(crate) fn hardcoded_declare_names() -> HashSet<String> {
        let mut s = HashSet::new();
        s.insert("printf".to_string());
        s.insert("puts".to_string());
        s.insert("llvm.trap".to_string());
        s.insert("malloc".to_string());
        s.insert("realloc".to_string());
        s.insert("free".to_string());
        s.insert("llvm.memcpy.p0i8.p0i8.i64".to_string());
        s.insert("xiom_is_sorted".to_string());
        s.insert("xiom_all".to_string());
        s.insert("xiom_none".to_string());
        s.insert("xiom_contains".to_string());
        s.insert("xiom_read_file".to_string());
        s.insert("xiom_file_size".to_string());
        s.insert("xiom_free".to_string());
        s.insert("xiom_char_at".to_string());
        s.insert("xiom_str_len".to_string());
        s.insert("xiom_str_concat".to_string());
        s.insert("xiom_str_slice".to_string());
        s.insert("xiom_str_starts_with".to_string());
        s.insert("xiom_str_ends_with".to_string());
        s.insert("xiom_int_to_string".to_string());
        s.insert("xiom_intern".to_string());
        s.insert("xiom_lookup".to_string());
        s.insert("xiom_ir_open".to_string());
        s.insert("xiom_ir_close".to_string());
        s.insert("xiom_ir_header".to_string());
        s.insert("xiom_ir_define".to_string());
        s.insert("xiom_ir_param".to_string());
        s.insert("xiom_ir_entry".to_string());
        s.insert("xiom_ir_alloca".to_string());
        s.insert("xiom_ir_store".to_string());
        s.insert("xiom_ir_load".to_string());
        s.insert("xiom_ir_binop".to_string());
        s.insert("xiom_ir_call".to_string());
        s.insert("xiom_ir_call_arg".to_string());
        s.insert("xiom_ir_call_lit".to_string());
        s.insert("xiom_ir_call_end".to_string());
        s.insert("xiom_ir_ret".to_string());
        s.insert("xiom_ir_ret_void".to_string());
        s.insert("xiom_ir_endfn".to_string());
        s.insert("xiom_ir_raw".to_string());
        s.insert("xiom_ir_emit_program".to_string());
        s.insert("xiom_ir_define_s".to_string());
        s.insert("xiom_ir_param_int".to_string());
        s.insert("xiom_ir_param_double".to_string());
        s.insert("xiom_ir_alloca_s".to_string());
        s.insert("xiom_ir_store_param".to_string());
        s.insert("xiom_ir_load_s".to_string());
        s.insert("xiom_ir_add".to_string());
        s.insert("xiom_ir_fmul".to_string());
        s.insert("xiom_ir_call_fn".to_string());
        s.insert("xiom_ir_call_arg_lit".to_string());
        s.insert("xiom_ir_ret_reg".to_string());
        s.insert("xiom_ir_ret_lit".to_string());
        s.insert("xiom_fn_table_init".to_string());
        s.insert("xiom_set_source".to_string());
        s.insert("xiom_fn_table_add".to_string());
        s.insert("xiom_fn_table_count".to_string());
        s.insert("xiom_fn_name_id".to_string());
        s.insert("xiom_fn_ret_type_id".to_string());
        s.insert("xiom_fn_param_count".to_string());
        s.insert("xiom_fn_body_start".to_string());
        s.insert("xiom_fn_body_end".to_string());
        s.insert("xiom_fn_emit_all".to_string());
        // strcmp is declared in emit_metadata_tables (conditional)
        s.insert("strcmp".to_string());
        s
    }

    /// Returns true if any `extern "C"` block in `items` (recursively through
    /// modules) declares a function named `name`.
    pub(crate) fn program_declares_extern(items: &[TopDecl], name: &str) -> bool {
        for item in items {
            match item {
                TopDecl::Extern(eb) => {
                    if eb.functions.iter().any(|f| f.name.name == name) {
                        return true;
                    }
                }
                TopDecl::Module(md) => {
                    if IrEmitter::program_declares_extern(&md.items, name) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Recursively collect `(name, requires_count, ensures_count)` for every
    /// function that carries at least one pre/postcondition, in source order.
    pub(crate) fn collect_contract_fns(items: &[TopDecl], out: &mut Vec<(String, usize, usize)>) {
        for item in items {
            match item {
                TopDecl::Fn(fd) => {
                    if !fd.contracts.is_empty() {
                        let mut pre = 0usize;
                        let mut post = 0usize;
                        for c in &fd.contracts {
                            match c {
                                ContractClause::Requires(_, _) => pre += 1,
                                ContractClause::Ensures(_, _) => post += 1,
                            }
                        }
                        let name = if let Some(recv) = &fd.receiver {
                            format!("{}.{}", recv.name, fd.name.name)
                        } else {
                            fd.name.name.clone()
                        };
                        out.push((name, pre, post));
                    }
                }
                TopDecl::Module(md) => {
                    IrEmitter::collect_contract_fns(&md.items, out);
                }
                _ => {}
            }
        }
    }

    /// Escape a Rust string for embedding in an LLVM `c"..."` byte string,
    /// matching the convention already used for contract/display strings.
    pub(crate) fn escape_ir_string(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\22")
    }
}
