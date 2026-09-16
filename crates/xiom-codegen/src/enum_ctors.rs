// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::IrEmitter;
use xiom_ast::Expr;

impl IrEmitter {
    /// Compile an enum variant constructor like `JsonValue.Integer(42)`.
    /// Generates the struct literal with discriminant set to the variant index
    /// and payload fields populated from the constructor arguments.
    pub(crate) fn compile_enum_constructor(&mut self, enum_name: &str, variant_name: &str, args: &[Expr]) -> Result<(String, String), String> {
        let struct_ty = format!("%struct.{enum_name}");
        let alloca = self.fresh_tmp();
        self.emitln(&format!("  {alloca} = alloca {struct_ty}"));

        // Set discriminant (field 0) to variant index
        let var_idx = self.types.enum_variants.get(&enum_name.to_string())
            .and_then(|vars| vars.iter().position(|(v, _)| v == variant_name))
            .unwrap_or(0) as i64;
        let disc_gep = self.fresh_tmp();
        self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
        self.emitln(&format!("  store i64 {var_idx}, i64* {disc_gep}"));

        // Get the variant's field names and the parent enum's field list
        let parent_fields = self.types.types.get(&enum_name.to_string()).unwrap_or_default();
        let variant_fields = self.types.enum_variants.get(&enum_name.to_string())
            .and_then(|vars| vars.into_iter().find(|(v, _)| v == variant_name))
            .map(|(_, vf)| vf.clone())
            .unwrap_or_default();

        // Store constructor args into the corresponding enum fields
        let arg_count = std::cmp::min(args.len(), variant_fields.len());
        for i in 0..arg_count {
            let (val, val_ty) = self.compile_expr(&args[i])?;
            let field_name = &variant_fields[i];
            // Find the field index in the parent enum's field list
            let field_idx = parent_fields.iter().position(|f| f == field_name).unwrap_or(i + 1);
            let field_llvm_ty = self.field_llvm_type(enum_name, field_idx);
            // 5c.29: float payloads are stored as RAW BITS in the i64 slot
            // (matching Some(x)/Vec-element conventions via val_to_i64) so
            // readers can bit-reinterpret. coerce_value would fptosi and
            // destroy the fraction (Real(2.718) became 2).
            // 5c.30: by-value STRUCT payloads (Vec headers, nested structs)
            // are heap-boxed via val_to_i64 (malloc+store+ptrtoint) so the
            // i64 slot holds a stable handle -- coerce_value extracted only
            // the FIRST FIELD (JsonValue.Array held Vec.data, not a header).
            let store_val = if field_llvm_ty == "i64"
                && (val_ty == "double" || val_ty == "float"
                    || (val_ty.starts_with("%struct.") && !val_ty.ends_with('*')))
            {
                self.val_to_i64(&val, &val_ty)
            } else {
                self.coerce_value(&val, &val_ty, &field_llvm_ty)
            };
            let gep = self.fresh_tmp();
            self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {field_idx}"));
            self.emitln(&format!("  store {field_llvm_ty} {store_val}, {field_llvm_ty}* {gep}"));
        }

        let loaded = self.fresh_tmp();
        self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {alloca}"));
        Ok((loaded, struct_ty))
    }
}
