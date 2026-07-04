    fn compile_invariant_call(&mut self, type_name: &str, struct_val_reg: &str) {
        let meta = match self.type_meta.get(type_name) {
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

    // ========================================================================
    // Derive Code Generation
    // ========================================================================

    fn compile_derive_impls(&mut self, items: &[TopDecl]) -> Result<(), String> {
        for item in items {
            self.compile_derive_for_item(item)?;
        }
        Ok(())
    }

    fn compile_derive_for_item(&mut self, item: &TopDecl) -> Result<(), String> {
        match item {
            TopDecl::Type(td) => {
                let bare_name = &td.name.name;
                // Resolve to qualified name using module context
                let type_name = if let Some(ref module) = self.current_module {
                    let qualified = format!("{}.{}", module, bare_name);
                    if self.type_meta.contains_key(&qualified) { qualified } else { bare_name.clone() }
                } else {
                    bare_name.clone()
                };
                let field_names: Vec<String> = td.fields.iter().map(|f| f.name.name.clone()).collect();
                let struct_ty = self.llvm_type_for(&type_name)?;

                for derive in &td.derives {
                    match derive {
                        DeriveTrait::Eq => self.compile_eq_impl(&type_name, &struct_ty, &field_names, &td.fields)?,
                        DeriveTrait::Clone => self.compile_clone_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Display => self.compile_display_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Hash => self.compile_hash_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Ord => self.compile_ord_impl(&type_name, &struct_ty, &field_names, &td.fields)?,
                    }
                }

                // Generate invariant check function if needed (even without derives)
                if !td.invariants.is_empty() {
                    self.compile_invariant_check(&type_name)?;
                }
            }
            TopDecl::Enum(ed) => {
                if ed.derives.is_empty() {
                    return Ok(());
                }
                let bare_name = &ed.name.name;
                let type_name = if let Some(ref module) = self.current_module {
                    let qualified = format!("{}.{}", module, bare_name);
                    if self.type_meta.contains_key(&qualified) { qualified } else { bare_name.clone() }
                } else {
                    bare_name.clone()
                };
                if !self.types.contains_key(&type_name) {
                    self.types.insert(type_name.clone(), vec!["discriminant".to_string()]);
                    self.type_meta.insert(type_name.clone(), TypeMeta {
                        fields: vec![("discriminant".to_string(), "Int".to_string())],
                        derives: ed.derives.clone(),
                        invariants: Vec::new(),
                    });
                }
                let struct_ty = self.llvm_type_for(&type_name)?;
                let field_names: Vec<String> = vec!["discriminant".to_string()];

                for derive in &ed.derives {
                    match derive {
                        DeriveTrait::Eq => self.compile_eq_impl(&type_name, &struct_ty, &field_names, &[])?,
                        DeriveTrait::Clone => self.compile_clone_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Hash => self.compile_hash_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Ord => self.compile_ord_impl(&type_name, &struct_ty, &field_names, &[])?,
                        _ => {}
                    }
                }
            }
            TopDecl::Module(md) => {
                let saved_module = self.current_module.clone();
                self.current_module = Some(if let Some(ref prev) = saved_module {
                    format!("{}.{}", prev, md.name.name)
                } else {
                    md.name.name.clone()
                });
                for sub in &md.items {
                    self.compile_derive_for_item(sub)?;
                }
                self.current_module = saved_module;
            }
            _ => {}
        }
        Ok(())
    }

    fn compile_eq_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String], _fields: &[FieldDecl]) -> Result<(), String> {
        let fn_name = format!("{type_name}.eq");
        if self.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.emitted_fns.insert(fn_name.clone());
        self.functions.insert(fn_name.clone(), (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));
        self.emitln(&format!("define i64 @{fn_name}({struct_ty} %self, {struct_ty} %other) {{"));
        let self_alloca = self.fresh_tmp();
        let other_alloca = self.fresh_tmp();
        self.emitln(&format!("  {self_alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %self, {struct_ty}* {self_alloca}"));
        self.emitln(&format!("  {other_alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %other, {struct_ty}* {other_alloca}"));

        let mut last_cmp = String::new();
        for (i, _fname) in field_names.iter().enumerate() {
            let self_gep = self.fresh_tmp();
            let self_val = self.fresh_tmp();
            let other_gep = self.fresh_tmp();
            let other_val = self.fresh_tmp();
            let cmp = self.fresh_tmp();
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            self.emitln(&format!("  {self_gep} = getelementptr {struct_ty}, {struct_ty}* {self_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {self_val} = load {field_llvm_ty}, {field_llvm_ty}* {self_gep}"));
            self.emitln(&format!("  {other_gep} = getelementptr {struct_ty}, {struct_ty}* {other_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {other_val} = load {field_llvm_ty}, {field_llvm_ty}* {other_gep}"));
            let is_float = field_llvm_ty == "double";
            if field_llvm_ty.starts_with("%struct.") {
                let field_type_name = &field_llvm_ty[8..];
                let eq_fn = format!("{field_type_name}.eq");
                self.emitln(&format!("  {cmp} = call i64 @{eq_fn}({field_llvm_ty} {self_val}, {field_llvm_ty} {other_val})"));
                if last_cmp.is_empty() {
                    last_cmp = cmp;
                } else {
                    let and_tmp = self.fresh_tmp();
                    self.emitln(&format!("  {and_tmp} = and i64 {last_cmp}, {cmp}"));
                    last_cmp = and_tmp;
                }
            } else if is_float {
                self.emitln(&format!("  {cmp} = fcmp oeq {field_llvm_ty} {self_val}, {other_val}"));
                let ze = self.fresh_tmp();
                self.emitln(&format!("  {ze} = zext i1 {cmp} to i64"));
                if last_cmp.is_empty() {
                    last_cmp = ze;
                } else {
                    let and_tmp = self.fresh_tmp();
                    self.emitln(&format!("  {and_tmp} = and i64 {last_cmp}, {ze}"));
                    last_cmp = and_tmp;
                }
            } else {
                self.emitln(&format!("  {cmp} = icmp eq {field_llvm_ty} {self_val}, {other_val}"));
                let ze = self.fresh_tmp();
                self.emitln(&format!("  {ze} = zext i1 {cmp} to i64"));
                if last_cmp.is_empty() {
                    last_cmp = ze;
                } else {
                    let and_tmp = self.fresh_tmp();
                    self.emitln(&format!("  {and_tmp} = and i64 {last_cmp}, {ze}"));
                    last_cmp = and_tmp;
                }
            }
        }
        if last_cmp.is_empty() {
            self.emitln("  ret i64 1");
        } else {
            self.emitln(&format!("  ret i64 {last_cmp}"));
        }
        self.emitln("}\n");
        // Register the generated function
        self.functions.insert(fn_name, (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));
        Ok(())
    }

    fn compile_clone_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String]) -> Result<(), String> {
        let fn_name = format!("{type_name}.clone");
        if self.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.emitted_fns.insert(fn_name.clone());
        self.functions.insert(fn_name.clone(), (vec![struct_ty.to_string()], struct_ty.to_string()));
        self.emitln(&format!("define {struct_ty} @{fn_name}({struct_ty} %self) {{"));
        let self_alloca = self.fresh_tmp();
        let result_alloca = self.fresh_tmp();
        self.emitln(&format!("  {self_alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %self, {struct_ty}* {self_alloca}"));
        self.emitln(&format!("  {result_alloca} = alloca {struct_ty}"));

        for (i, fname) in field_names.iter().enumerate() {
            let src_gep = self.fresh_tmp();
            let src_val = self.fresh_tmp();
            let dst_gep = self.fresh_tmp();
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            self.emitln(&format!("  {src_gep} = getelementptr {struct_ty}, {struct_ty}* {self_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {src_val} = load {field_llvm_ty}, {field_llvm_ty}* {src_gep}"));
            self.emitln(&format!("  {dst_gep} = getelementptr {struct_ty}, {struct_ty}* {result_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  store {field_llvm_ty} {src_val}, {field_llvm_ty}* {dst_gep}"));
            let _ = fname;
        }
        let loaded = self.fresh_tmp();
        self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {result_alloca}"));
        self.emitln(&format!("  ret {struct_ty} {loaded}"));
        self.emitln("}\n");
        self.functions.insert(fn_name, (vec![struct_ty.to_string()], struct_ty.to_string()));
        Ok(())
    }

    fn compile_display_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String]) -> Result<(), String> {
        let fn_name = format!("{type_name}.to_str");
        if self.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.emitted_fns.insert(fn_name.clone());
        self.functions.insert(fn_name.clone(), (vec![struct_ty.to_string()], "i8*".to_string()));
        self.emitln(&format!("define i8* @{fn_name}({struct_ty} %self) {{"));
        let self_alloca = self.fresh_tmp();
        self.emitln(&format!("  {self_alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %self, {struct_ty}* {self_alloca}"));

        // Build format string: "TypeName{ field1: ..., field2: ... }"
        let mut display_parts: Vec<String> = vec![format!("{type_name}{{")];
        for fname in field_names {
            display_parts.push(format!("{fname}: "));
            display_parts.push("%lld ".to_string());
        }
        display_parts.push("}".to_string());
        let fmt_str = display_parts.concat();
        let fmt_label = format!("@.fmt_{fn_name}");
        let escaped = fmt_str.replace('\\', "\\\\").replace('"', "\\22")
            .replace('\n', "\\0A").replace('\t', "\\09");
        self.strings.push(format!(
            "{fmt_label} = private unnamed_addr constant [{len} x i8] c\"{escaped}\\00\"",
            len = fmt_str.len() + 1
        ));

        // Allocate output buffer (256 bytes fixed)
        let buf = self.fresh_tmp();
        self.emitln(&format!("  {buf} = alloca i8, i64 256"));

        // Build sprintf call
        let fmt_ptr = self.fresh_tmp();
        self.emitln(&format!("  {fmt_ptr} = getelementptr [{len} x i8], [{len} x i8]* {fmt_label}, i64 0, i64 0",
            len = fmt_str.len() + 1));

        let mut args = vec![format!("i8* {fmt_ptr}")];
        for (i, _) in field_names.iter().enumerate() {
            let gep = self.fresh_tmp();
            let val = self.fresh_tmp();
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {self_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {val} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
            args.push(format!("{field_llvm_ty} {val}"));
        }
        let buf_ptr = self.fresh_tmp();
        self.emitln(&format!("  {buf_ptr} = getelementptr i8, i8* {buf}, i64 0"));
        self.emitln(&format!("  call i32 (i8*, ...) @printf(i8* {buf_ptr})"));

        // For Phase 1, just return a pointer to the buf (simplified)
        self.emitln(&format!("  ret i8* {buf_ptr}"));
        self.emitln("}\n");
        self.functions.insert(fn_name, (vec![struct_ty.to_string()], "i8*".to_string()));
        Ok(())
    }

    fn compile_hash_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String]) -> Result<(), String> {
        let fn_name = format!("{type_name}.hash");
        if self.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.emitted_fns.insert(fn_name.clone());
        self.functions.insert(fn_name.clone(), (vec![struct_ty.to_string()], "i64".to_string()));
        self.emitln(&format!("define i64 @{fn_name}({struct_ty} %self) {{"));
        let self_alloca = self.fresh_tmp();
        self.emitln(&format!("  {self_alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %self, {struct_ty}* {self_alloca}"));

        // Seed: 5381
        self.emitln("  %hash = alloca i64");
        self.emitln("  store i64 5381, i64* %hash");

        for (i, _) in field_names.iter().enumerate() {
            let gep = self.fresh_tmp();
            let val = self.fresh_tmp();
            let loaded_hash = self.fresh_tmp();
            let mul_tmp = self.fresh_tmp();
            let add_tmp = self.fresh_tmp();
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {self_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {val} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
            self.emitln(&format!("  {loaded_hash} = load i64, i64* %hash"));
            self.emitln(&format!("  {mul_tmp} = mul i64 {loaded_hash}, 33"));
            self.emitln(&format!("  {add_tmp} = add i64 {mul_tmp}, {val}"));
            self.emitln(&format!("  store i64 {add_tmp}, i64* %hash"));
        }
        let final_hash = self.fresh_tmp();
        self.emitln(&format!("  {final_hash} = load i64, i64* %hash"));
        self.emitln(&format!("  ret i64 {final_hash}"));
        self.emitln("}\n");
        self.functions.insert(fn_name, (vec![struct_ty.to_string()], "i64".to_string()));
        Ok(())
    }

    fn compile_ord_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String], _fields: &[FieldDecl]) -> Result<(), String> {
        let fn_name = format!("{type_name}.compare");
        if self.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.emitted_fns.insert(fn_name.clone());
        self.functions.insert(fn_name.clone(), (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));
        self.emitln(&format!("define i64 @{fn_name}({struct_ty} %self, {struct_ty} %other) {{"));
        let self_alloca = self.fresh_tmp();
        let other_alloca = self.fresh_tmp();
        self.emitln(&format!("  {self_alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %self, {struct_ty}* {self_alloca}"));
        self.emitln(&format!("  {other_alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %other, {struct_ty}* {other_alloca}"));

        for i in 0..field_names.len() {
            let self_gep = self.fresh_tmp();
            let self_val = self.fresh_tmp();
            let other_gep = self.fresh_tmp();
            let other_val = self.fresh_tmp();
            let cmp_eq = self.fresh_tmp();
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            let is_float = field_llvm_ty == "double";
            self.emitln(&format!("  {self_gep} = getelementptr {struct_ty}, {struct_ty}* {self_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {self_val} = load {field_llvm_ty}, {field_llvm_ty}* {self_gep}"));
            self.emitln(&format!("  {other_gep} = getelementptr {struct_ty}, {struct_ty}* {other_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {other_val} = load {field_llvm_ty}, {field_llvm_ty}* {other_gep}"));

            let next_field = self.fresh_block("next_field");
            let ret_block = self.fresh_block("ord_ret");
            if field_llvm_ty.starts_with("%struct.") {
                let field_type_name = &field_llvm_ty[8..];
                let compare_fn = format!("{field_type_name}.compare");
                let cmp_result = self.fresh_tmp();
                self.emitln(&format!("  {cmp_result} = call i64 @{compare_fn}({field_llvm_ty} {self_val}, {field_llvm_ty} {other_val})"));
                self.emitln(&format!("  {cmp_eq} = icmp eq i64 {cmp_result}, 0"));
                self.emitln(&format!("  br i1 {cmp_eq}, label %{next_field}, label %{ret_block}"));
                self.emitln(&format!("\n{ret_block}:"));
                self.emitln(&format!("  ret i64 {cmp_result}"));
            } else if is_float {
                self.emitln(&format!("  {cmp_eq} = fcmp oeq {field_llvm_ty} {self_val}, {other_val}"));
                self.emitln(&format!("  br i1 {cmp_eq}, label %{next_field}, label %{ret_block}"));
                self.emitln(&format!("\n{ret_block}:"));
                let fcmp = self.fresh_tmp();
                self.emitln(&format!("  {fcmp} = fcmp olt {field_llvm_ty} {self_val}, {other_val}"));
                let result = self.fresh_tmp();
                self.emitln(&format!("  {result} = select i1 {fcmp}, i64 -1, i64 1"));
                self.emitln(&format!("  ret i64 {result}"));
            } else {
                self.emitln(&format!("  {cmp_eq} = icmp eq {field_llvm_ty} {self_val}, {other_val}"));
                self.emitln(&format!("  br i1 {cmp_eq}, label %{next_field}, label %{ret_block}"));
                self.emitln(&format!("\n{ret_block}:"));
                let cmp_lt = self.fresh_tmp();
                self.emitln(&format!("  {cmp_lt} = icmp slt {field_llvm_ty} {self_val}, {other_val}"));
                let result = self.fresh_tmp();
                self.emitln(&format!("  {result} = select i1 {cmp_lt}, i64 -1, i64 1"));
                self.emitln(&format!("  ret i64 {result}"));
            }
            self.emitln(&format!("\n{next_field}:"));
        }
        self.emitln("  ret i64 0");
        self.emitln("}\n");
        self.functions.insert(fn_name, (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));
        Ok(())
    }

    // ========================================================================
    // Generics: Monomorphisation
    // ========================================================================

    fn monomorphised_fn_name(&self, base_name: &str, concrete_types: &[String]) -> String {
        if concrete_types.is_empty() {
            base_name.to_string()
        } else {
            format!("{}_{}", base_name, concrete_types.join("_"))
        }
    }

    /// After all non-generic functions have been compiled, emit specialized
    /// versions for each tracked generic instantiation.
    /// Uses a worklist pattern: monomorphising one function may trigger new
    /// instantiations (generic chains), which are processed in subsequent passes.
    fn compile_generic_monomorphisations(&mut self) -> Result<(), String> {
        let mut iteration: u32 = 0;
        const MAX_GENERIC_ITERATIONS: u32 = 256;
        loop {
            iteration += 1;
            if iteration > MAX_GENERIC_ITERATIONS {
                return Err(format!(
                    "generic monomorphisation exceeded {} iterations — possible infinite recursion in generic definitions",
                    MAX_GENERIC_ITERATIONS
                ));
            }
            let instantiations = std::mem::take(&mut self.generic_instantiations);
            if instantiations.is_empty() {
                break;
            }
            for (base_name, concrete_types) in &instantiations {
            // Find the generic function decl
            let fd = match self.generic_fn_decls.iter().find(|f| self.fn_key(f) == *base_name) {
                Some(f) => f.clone(),
                None => continue,
            };
            // Check interface bounds for each generic parameter
            for (gp, concrete_type) in fd.generics.iter().zip(concrete_types.iter()) {
                for bound in &gp.bounds {
                    if let Some(methods) = self.interfaces.get(&bound.name) {
                        for (method_name, _) in methods {
                            let method_key = format!("{}.{}", concrete_type, method_name);
                            if !self.functions.contains_key(&method_key) {
                                return Err(format!(
                                    "type '{}' does not implement '{}': missing method '{}'",
                                    concrete_type, bound.name, method_name
                                ));
                            }
                        }
                    }
                }
            }
            let specialized_name = self.monomorphised_fn_name(base_name, concrete_types);
            // Build type substitution map: generic param name -> concrete type name
            let mut type_map: HashMap<String, String> = HashMap::new();
            for (gp, ct) in fd.generics.iter().zip(concrete_types.iter()) {
                type_map.insert(gp.name.name.clone(), ct.clone());
            }
            // Register concrete tuple types for this monomorphisation
            if let Some(ref ret_ty) = fd.return_type {
                self.ensure_concrete_tuple_type_registered(ret_ty, &type_map);
            }
            for p in &fd.params {
                self.ensure_concrete_tuple_type_registered(&p.ty, &type_map);
            }
            // Register the specialized function signature
            let struct_types: HashSet<String> = self.types.keys().cloned().collect();
            let subst_type = |t: &Type| -> String {
                match t {
                    Type::Named(id, _) => {
                        let raw = if let Some(ct) = type_map.get(&id.name) {
                            ct.clone()
                        } else {
                            id.name.clone()
                        };
                        if struct_types.contains(&raw) {
                            format!("%struct.{raw}")
                        } else {
                            Self::xiom_to_llvm_type(&raw).to_string()
                        }
                    }
                    Type::Tuple(elems) => {
                        let parts: Vec<String> = elems.iter().map(|e| {
                            // Resolve element types: substitute generics, then resolve to LLVM name
                            let xiom_name = match e {
                                Type::Named(id, _) => type_map.get(&id.name).cloned().unwrap_or_else(|| id.name.clone()),
                                _ => Self::type_from_ast(e),
                            };
                            // Strip %struct. prefix if present (element might already be a struct type)
                            if let Some(stripped) = xiom_name.strip_prefix("%struct.") {
                                stripped.to_string()
                            } else {
                                xiom_name
                            }
                        }).collect();
                        format!("%struct.Tuple_{}", parts.join("_"))
                    }
                    _ => Self::xiom_to_llvm_type(&Self::type_from_ast(t)).to_string(),
                }
            };
            let specialized_ret_type = fd.return_type.as_ref()
                .map(|t| subst_type(t))
                .unwrap_or_else(|| "void".to_string());
            let mut specialized_param_types: Vec<String> = Vec::new();
            // Include self/receiver parameter for methods
            let self_llvm_ty = if let Some(ref r) = fd.receiver {
                Some(self.llvm_type_for(&r.name)?)
            } else {
                None
            };
            if let Some(ref st) = self_llvm_ty {
                specialized_param_types.push(st.clone());
            }
            let explicit_param_types: Vec<String> = fd.params.iter()
                .map(|p| subst_type(&p.ty))
                .collect();
            specialized_param_types.extend(explicit_param_types);
            self.functions.insert(specialized_name.clone(), (specialized_param_types.clone(), specialized_ret_type.clone()));

            // Emit the specialized function
            self.push_scope();
            self.block_counter = 0;
            self.tmp_counter = 0;

            self.current_return_type = specialized_ret_type.clone();
            self.current_param_llvm_types = specialized_param_types.clone();
            self.current_fn = Some(specialized_name.clone());

            let self_offset: usize = if self_llvm_ty.is_some() { 1 } else { 0 };
            let mut params_str: Vec<String> = Vec::new();
            if let Some(ref st) = self_llvm_ty {
                params_str.push(format!("{st} %param_self"));
            }
            let explicit_params_str: Vec<String> = fd.params.iter()
                .enumerate()
                .map(|(i, p)| {
                    let llvm_ty = subst_type(&p.ty);
                    format!("{llvm_ty} %param{}", i + self_offset)
                })
                .collect();
            params_str.extend(explicit_params_str);

            self.emitln(&format!("define {specialized_ret_type} @{specialized_name}({}) {{", params_str.join(", ")));
            let entry_block = self.fresh_block("entry");
            self.emitln(&format!("{entry_block}:"));

            // Allocate parameters as locals
            // Allocate self parameter first (for methods)
            if let (Some(st), Some(recv)) = (&self_llvm_ty, &fd.receiver) {
                let self_alloca = self.fresh_tmp();
                self.emitln(&format!("  {self_alloca} = alloca {st}"));
                self.emitln(&format!("  store {st} %param_self, {st}* {self_alloca}"));
                self.add_local("self", self_alloca.clone(), st);
                // Register each struct field as a local (bare name access like `items`)
                let recv_type_name = &recv.name;
                let names_opt = self.types.get(recv_type_name).cloned()
                    .or_else(|| {
                        // Try module-qualified variant
                        if let Some(ref module) = self.current_module {
                            let qualified = format!("{}.{}", module, recv_type_name);
                            self.types.get(&qualified).cloned()
                        } else {
                            // Search all keys
                            self.types.iter()
                                .find(|(k, _)| k.ends_with(&format!(".{recv_type_name}")))
                                .map(|(_, v)| v.clone())
                        }
                    });
                if let Some(names) = names_opt {
                    // Use the found type key for field_llvm_type lookups
                    let type_key = self.types.get(recv_type_name).map(|_| recv_type_name.clone())
                        .or_else(|| {
                            if let Some(ref module) = self.current_module {
                                let q = format!("{}.{}", module, recv_type_name);
                                if self.types.contains_key(&q) { Some(q) } else { None }
                            } else { None }
                        })
                        .or_else(|| {
                            self.types.keys().find(|k| k.ends_with(&format!(".{recv_type_name}"))).cloned()
                        })
                        .unwrap_or_else(|| recv_type_name.clone());
                    for (idx, field_name) in names.iter().enumerate() {
                        let field_llvm_ty = self.field_llvm_type(&type_key, idx);
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr {st}, {st}* {self_alloca}, i32 0, i32 {idx}"));
                        self.add_local(field_name, gep, &field_llvm_ty);
                    }
                }
            }
            for (i, param) in fd.params.iter().enumerate() {
                let llvm_ty = subst_type(&param.ty);
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
                let param_idx = i + self_offset;
                self.emitln(&format!("  store {llvm_ty} %param{param_idx}, {llvm_ty}* {alloca}"));
                self.add_local(&param.name.name, alloca, &llvm_ty);
                // Track params whose original type is a generic parameter being monomorphised
                let xiom_ty = Self::type_from_ast(&param.ty);
                if type_map.contains_key(&xiom_ty) {
                    if let Some(concrete) = type_map.get(&xiom_ty) {
                        self.param_concrete_types.insert(param.name.name.clone(), concrete.clone());
                    }
                }
            }

            // Set type substitution map for method dispatch in body
            self.current_type_map = type_map.clone();

            // Compile body
            if let Some(body) = fd.body.as_ref() {
                self.compile_block(body, fd.return_type.is_some())?;
            }

            // Clear type substitution state
            self.current_type_map.clear();
            self.param_concrete_types.clear();
            if fd.return_type.is_none() {
                self.emitln("  ret void");
            }
                self.emitln("}\n");
                self.pop_scope();
                self.current_fn = None;
            }
        }
        Ok(())
    }
