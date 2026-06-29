//! AXIOM Type Checker — Phase 0: basic type checking for primitives,
//! struct types, function signatures, and return types.
//! No generics, no ownership, no contracts enforcement.

use axiom_ast::*;
use std::collections::HashMap;

// ============================================================================
// Type representation for the checker
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum CheckedType {
    Bool,
    Int, Int8, Int16, Int32, Int64,
    UInt, UInt8, UInt16, UInt32, UInt64,
    Float32, Float64,
    Char,
    Str,
    Unit,
    Never,
    /// A user-defined type by name
    Named(String),
    /// A generic type parameter (still unresolved)
    Generic(String),
    /// Error type — used when type checking fails
    Error,
}

impl CheckedType {
    /// Convert from AST Type to checked type representation
    pub fn from_ast_type(ty: &Type) -> Self {
        match ty {
            Type::Named(ident, _) => Self::from_str(&ident.name),
            Type::Ref(inner) => CheckedType::from_ast_type(inner),
            Type::MutRef(inner) => CheckedType::from_ast_type(inner),
            Type::Option(_) => CheckedType::Named("Option".into()),
            Type::Result(_, _) => CheckedType::Named("Result".into()),
            Type::Vec(_) => CheckedType::Named("Vec".into()),
            Type::Slice(_) => CheckedType::Named("Slice".into()),
            Type::Map(_, _) => CheckedType::Named("Map".into()),
            Type::Set(_) => CheckedType::Named("Set".into()),
            Type::Tuple(types) => {
                // In Phase 0, represent tuples as Named for simplicity
                CheckedType::Named(format!("Tuple{}", types.len()))
            }
            Type::Ptr(_) => CheckedType::Named("Ptr".into()),
            Type::Array(_, _) => CheckedType::Named("Array".into()),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Bool" => CheckedType::Bool,
            "Int" => CheckedType::Int,
            "Int8" => CheckedType::Int8,
            "Int16" => CheckedType::Int16,
            "Int32" => CheckedType::Int32,
            "Int64" => CheckedType::Int64,
            "UInt" => CheckedType::UInt,
            "UInt8" => CheckedType::UInt8,
            "UInt16" => CheckedType::UInt16,
            "UInt32" => CheckedType::UInt32,
            "UInt64" => CheckedType::UInt64,
            "Float32" => CheckedType::Float32,
            "Float64" => CheckedType::Float64,
            "Char" => CheckedType::Char,
            "Str" => CheckedType::Str,
            "()" => CheckedType::Unit,
            _ => CheckedType::Named(s.to_string()),
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self,
            CheckedType::Int | CheckedType::Int8 | CheckedType::Int16 |
            CheckedType::Int32 | CheckedType::Int64 |
            CheckedType::UInt | CheckedType::UInt8 | CheckedType::UInt16 |
            CheckedType::UInt32 | CheckedType::UInt64 |
            CheckedType::Float32 | CheckedType::Float64
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(self,
            CheckedType::Int | CheckedType::Int8 | CheckedType::Int16 |
            CheckedType::Int32 | CheckedType::Int64 |
            CheckedType::UInt | CheckedType::UInt8 | CheckedType::UInt16 |
            CheckedType::UInt32 | CheckedType::UInt64
        )
    }

    pub fn name(&self) -> String {
        match self {
            CheckedType::Bool => "Bool".into(),
            CheckedType::Int => "Int".into(),
            CheckedType::Int8 => "Int8".into(),
            CheckedType::Int16 => "Int16".into(),
            CheckedType::Int32 => "Int32".into(),
            CheckedType::Int64 => "Int64".into(),
            CheckedType::UInt => "UInt".into(),
            CheckedType::UInt8 => "UInt8".into(),
            CheckedType::UInt16 => "UInt16".into(),
            CheckedType::UInt32 => "UInt32".into(),
            CheckedType::UInt64 => "UInt64".into(),
            CheckedType::Float32 => "Float32".into(),
            CheckedType::Float64 => "Float64".into(),
            CheckedType::Char => "Char".into(),
            CheckedType::Str => "Str".into(),
            CheckedType::Unit => "()".into(),
            CheckedType::Never => "!".into(),
            CheckedType::Named(s) => s.clone(),
            CheckedType::Generic(s) => s.clone(),
            CheckedType::Error => "<error>".into(),
        }
    }
}

// ============================================================================
// Type Checker
// ============================================================================

pub struct Checker {
    /// Known type names → their field types
    types: HashMap<String, HashMap<String, CheckedType>>,
    /// Known function signatures
    functions: HashMap<String, FnSig>,
    /// Current function return type
    current_return: Option<CheckedType>,
    /// Local variable types
    locals: Vec<HashMap<String, CheckedType>>,
    errors: Vec<CheckError>,
}

#[derive(Debug, Clone)]
struct FnSig {
    params: Vec<(String, CheckedType)>,
    return_type: Option<CheckedType>,
}

#[derive(Debug, Clone)]
pub struct CheckError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type error at {}: {}", self.span, self.message)
    }
}

impl Checker {
    pub fn new() -> Self {
        let mut checker = Self {
            types: HashMap::new(),
            functions: HashMap::new(),
            current_return: None,
            locals: vec![HashMap::new()],
            errors: Vec::new(),
        };
        // Register built-in types
        checker.register_builtins();
        checker
    }

    fn register_builtins(&mut self) {
        // All primitive types are known
        for prim in &["Bool", "Int", "Int8", "Int16", "Int32", "Int64",
                       "UInt", "UInt8", "UInt16", "UInt32", "UInt64",
                       "Float32", "Float64", "Char", "Str"] {
            self.types.insert(prim.to_string(), HashMap::new());
        }
    }

    fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.locals.pop();
    }

    fn add_local(&mut self, name: &str, ty: CheckedType) {
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    fn lookup_local(&self, name: &str) -> Option<&CheckedType> {
        for scope in self.locals.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    fn error(&mut self, message: impl Into<String>, span: Span) -> CheckedType {
        self.errors.push(CheckError { message: message.into(), span });
        CheckedType::Error
    }

    // ========================================================================
    // Program-level checking
    // ========================================================================

    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<CheckError>> {
        // Register all type declarations first
        for item in &program.items {
            self.register_type_decl(item);
        }

        // Register all function signatures
        for item in &program.items {
            self.register_fn_signature(item);
        }

        // Check all function bodies
        for item in &program.items {
            self.check_top_decl(item);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    fn register_type_decl(&mut self, item: &TopDecl) {
        match item {
            TopDecl::Type(td) => {
                let mut fields = HashMap::new();
                for field in &td.fields {
                    fields.insert(field.name.name.clone(), CheckedType::from_ast_type(&field.ty));
                }
                for (name, ty, _) in &td.derived_fields {
                    fields.insert(name.name.clone(), CheckedType::from_ast_type(ty));
                }
                self.types.insert(td.name.name.clone(), fields);
            }
            TopDecl::Enum(ed) => {
                // Enums are known types with no struct fields
                self.types.insert(ed.name.name.clone(), HashMap::new());
            }
            TopDecl::Module(md) => {
                for item in &md.items {
                    self.register_type_decl(item);
                }
            }
            _ => {}
        }
    }

    fn register_fn_signature(&mut self, item: &TopDecl) {
        match item {
            TopDecl::Fn(fd) => {
                let params: Vec<_> = fd.params.iter().map(|p| {
                    (p.name.name.clone(), CheckedType::from_ast_type(&p.ty))
                }).collect();
                let return_type = fd.return_type.as_ref().map(|t| CheckedType::from_ast_type(t));
                let key = if let Some(ref recv) = fd.receiver {
                    format!("{}.{}", recv.name, fd.name.name)
                } else {
                    fd.name.name.clone()
                };
                self.functions.insert(key, FnSig { params, return_type });
            }
            TopDecl::Module(md) => {
                for item in &md.items {
                    self.register_fn_signature(item);
                }
            }
            _ => {}
        }
    }

    fn check_top_decl(&mut self, item: &TopDecl) {
        match item {
            TopDecl::Fn(fd) => {
                if fd.body.is_some() {
                    self.check_fn_decl(fd);
                }
            }
            TopDecl::Module(md) => {
                for item in &md.items {
                    self.check_top_decl(item);
                }
            }
            TopDecl::Const(cd) => {
                let val_ty = self.check_expr(&cd.value);
                let decl_ty = CheckedType::from_ast_type(&cd.ty);
                if val_ty != CheckedType::Error && decl_ty != CheckedType::Error {
                    if !self.types_compatible(&val_ty, &decl_ty) {
                        self.error(
                            format!("const type mismatch: declared {}, found {}", decl_ty.name(), val_ty.name()),
                            cd.span,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    // ========================================================================
    // Function checking
    // ========================================================================

    fn check_fn_decl(&mut self, fd: &FnDecl) {
        self.push_scope();

        // Add parameters to scope
        for param in &fd.params {
            self.add_local(&param.name.name, CheckedType::from_ast_type(&param.ty));
        }

        // Set expected return type
        let expected_return = fd.return_type.as_ref().map(|t| CheckedType::from_ast_type(t));
        self.current_return = expected_return.clone();

        // Check body
        if let Some(ref body) = fd.body {
            self.check_block(body, expected_return);
        }

        self.pop_scope();
    }

    fn check_block(&mut self, block: &Block, expected_return: Option<CheckedType>) -> Option<CheckedType> {
        let mut last_expr_ty = None;

        for item in &block.stmts {
            match item {
                StmtOrExpr::Stmt(stmt) => {
                    self.check_stmt(stmt);
                }
                StmtOrExpr::Expr(expr) => {
                    last_expr_ty = Some(self.check_expr(expr));
                }
            }
        }

        // If this block is the function body, check return type
        if let Some(expected) = expected_return {
            if let Some(found) = &last_expr_ty {
                if found != &CheckedType::Error && expected != CheckedType::Error {
                    if !self.types_compatible(found, &expected) {
                        self.error(
                            format!("return type mismatch: expected {}, found {}", expected.name(), found.name()),
                            block.span,
                        );
                    }
                }
            } else if expected != CheckedType::Unit {
                // No expression at end, but return type expected
                // Only warn if there are no return statements (handled elsewhere)
            }
        }

        last_expr_ty
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(name, ty_annot, value, span) => {
                let val_ty = self.check_expr(value);
                if let Some(annot) = ty_annot {
                    let annot_ty = CheckedType::from_ast_type(annot);
                    if !self.types_compatible(&val_ty, &annot_ty) && val_ty != CheckedType::Error {
                        self.error(
                            format!("type mismatch in let: annotated {}, found {}", annot_ty.name(), val_ty.name()),
                            *span,
                        );
                    }
                }
                self.add_local(&name.name, val_ty);
            }
            Stmt::Var(name, ty_annot, value, span) => {
                let val_ty = self.check_expr(value);
                if let Some(annot) = ty_annot {
                    let annot_ty = CheckedType::from_ast_type(annot);
                    if !self.types_compatible(&val_ty, &annot_ty) && val_ty != CheckedType::Error {
                        self.error(
                            format!("type mismatch in var: annotated {}, found {}", annot_ty.name(), val_ty.name()),
                            *span,
                        );
                    }
                }
                self.add_local(&name.name, val_ty);
            }
            Stmt::Assign(place, value, span) => {
                let place_ty = self.check_expr(place);
                let val_ty = self.check_expr(value);
                if !self.types_compatible(&place_ty, &val_ty) && place_ty != CheckedType::Error && val_ty != CheckedType::Error {
                    self.error(
                        format!("assignment type mismatch: {} = {}", place_ty.name(), val_ty.name()),
                        *span,
                    );
                }
            }
            Stmt::Return(expr, span) => {
                let ret_ty = expr.as_ref().map(|e| self.check_expr(e)).unwrap_or(CheckedType::Unit);
                if let Some(ref expected) = self.current_return {
                    if !self.types_compatible(&ret_ty, expected) && ret_ty != CheckedType::Error {
                        self.error(
                            format!("return type mismatch: expected {}, found {}", expected.name(), ret_ty.name()),
                            *span,
                        );
                    }
                }
            }
            Stmt::Expr(expr, _) => {
                self.check_expr(expr);
            }
            Stmt::If(cond, then_block, elifs, else_block, _) => {
                let cond_ty = self.check_expr(cond);
                if cond_ty != CheckedType::Bool && cond_ty != CheckedType::Error {
                    self.error(format!("if condition must be Bool, found {}", cond_ty.name()), cond.span());
                }
                self.check_block(then_block, None);
                for (econd, eblock) in elifs {
                    let econd_ty = self.check_expr(econd);
                    if econd_ty != CheckedType::Bool && econd_ty != CheckedType::Error {
                        self.error(format!("elif condition must be Bool, found {}", econd_ty.name()), econd.span());
                    }
                    self.check_block(eblock, None);
                }
                if let Some(eb) = else_block {
                    self.check_block(eb, None);
                }
            }
            Stmt::Match(expr, arms, _) => {
                let matched_ty = self.check_expr(expr);
                for arm in arms {
                    let _ = &arm.pattern; // Phase 0: patterns aren't fully checked
                    match &arm.body {
                        MatchBody::Block(b) => { self.check_block(b, None); }
                        MatchBody::Expr(e) => { self.check_expr(e); }
                    }
                }
                let _ = matched_ty;
            }
            Stmt::While(cond, body, _) => {
                let cond_ty = self.check_expr(cond);
                if cond_ty != CheckedType::Bool && cond_ty != CheckedType::Error {
                    self.error(format!("while condition must be Bool, found {}", cond_ty.name()), cond.span());
                }
                self.check_block(body, None);
            }
            Stmt::For(var, iter, body, _) => {
                let _iter_ty = self.check_expr(iter);
                self.add_local(&var.name, CheckedType::Int); // simplified
                self.check_block(body, None);
            }
            Stmt::Spawn(body, _) => {
                self.check_block(body, None);
            }
        }
    }

    // ========================================================================
    // Expression type checking
    // ========================================================================

    fn check_expr(&mut self, expr: &Expr) -> CheckedType {
        match expr {
            Expr::Ident(ident) => {
                if let Some(ty) = self.lookup_local(&ident.name) {
                    ty.clone()
                } else {
                    // Could be a function name — look it up
                    if self.functions.contains_key(&ident.name) {
                        CheckedType::Named("fn".into())
                    } else {
                        self.error(format!("undefined variable '{}'", ident.name), ident.span)
                    }
                }
            }
            Expr::Int(_, _) => CheckedType::Int,
            Expr::Float(_, _) => CheckedType::Float64,
            Expr::Str(_, _) => CheckedType::Str,
            Expr::Char(_, _) => CheckedType::Char,
            Expr::Bool(_, _) => CheckedType::Bool,
            Expr::Paren(inner, _) => self.check_expr(inner),
            Expr::Unary(op, inner, span) => {
                let inner_ty = self.check_expr(inner);
                match op {
                    UnaryOp::Neg => {
                        if !inner_ty.is_numeric() {
                            self.error(format!("cannot negate type {}", inner_ty.name()), *span);
                        }
                        inner_ty
                    }
                    UnaryOp::Not => {
                        if inner_ty != CheckedType::Bool {
                            self.error(format!("cannot logically negate type {}", inner_ty.name()), *span);
                        }
                        CheckedType::Bool
                    }
                    UnaryOp::Ref | UnaryOp::MutRef => inner_ty, // reference keeps the type
                }
            }
            Expr::Binary(left, op, right, span) => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                        if !left_ty.is_numeric() {
                            self.error(format!("left operand must be numeric, found {}", left_ty.name()), *span);
                        }
                        if !right_ty.is_numeric() {
                            self.error(format!("right operand must be numeric, found {}", right_ty.name()), *span);
                        }
                        left_ty // result type is the left operand type (promotion in Phase 1)
                    }
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                        CheckedType::Bool
                    }
                    BinOp::And | BinOp::Or => {
                        if left_ty != CheckedType::Bool {
                            self.error(format!("left operand of logical op must be Bool, found {}", left_ty.name()), *span);
                        }
                        if right_ty != CheckedType::Bool {
                            self.error(format!("right operand of logical op must be Bool, found {}", right_ty.name()), *span);
                        }
                        CheckedType::Bool
                    }
                    BinOp::Assign => right_ty,
                }
            }
            Expr::Try(inner, _span) => {
                let inner_ty = self.check_expr(inner);
                // ? unwraps Result or Option — return the inner type
                // Phase 0 simplified: just pass through
                inner_ty
            }
            Expr::Imply(_, _, _) => CheckedType::Bool,
            Expr::Is(_, _, _) => CheckedType::Bool,
            Expr::Field(obj, field, span) => {
                let obj_ty = self.check_expr(obj);
                match &obj_ty {
                    CheckedType::Named(name) => {
                        if let Some(fields) = self.types.get(name) {
                            if let Some(field_ty) = fields.get(&field.name) {
                                field_ty.clone()
                            } else {
                                self.error(
                                    format!("type '{}' has no field '{}'", name, field.name),
                                    *span,
                                )
                            }
                        } else {
                            CheckedType::Error // unknown type
                        }
                    }
                    _ => self.error(
                        format!("cannot access field on non-struct type {}", obj_ty.name()),
                        *span,
                    ),
                }
            }
            Expr::Call(func, args, span) => {
                let _func_ty = self.check_expr(func);
                // Look up the function by name if it's a simple identifier
                if let Expr::Ident(ref name) = **func {
                    if let Some(sig) = self.functions.get(&name.name).cloned() {
                        for (i, arg) in args.iter().enumerate() {
                            let arg_ty = self.check_expr(arg);
                            if i < sig.params.len() {
                                let expected = &sig.params[i].1;
                                if !self.types_compatible(&arg_ty, expected) && arg_ty != CheckedType::Error {
                                    self.error(
                                        format!("argument {} type mismatch: expected {}, found {}",
                                            i + 1, expected.name(), arg_ty.name()),
                                        *span,
                                    );
                                }
                            }
                        }
                        return sig.return_type.unwrap_or(CheckedType::Unit);
                    }
                }
                // Fallback: could be a method call or unknown function
                CheckedType::Unit
            }
            Expr::Index(_, _, _) => CheckedType::Int, // simplified
            Expr::AtPre(inner, _) => self.check_expr(inner),
            Expr::Ref(inner, _) | Expr::MutRef(inner, _) => self.check_expr(inner),
            Expr::Some(inner, _) => {
                let _ = self.check_expr(inner);
                CheckedType::Named("Option".into())
            }
            Expr::None(_) => CheckedType::Named("Option".into()),
            Expr::Ok(inner, _) => {
                let _ = self.check_expr(inner);
                CheckedType::Named("Result".into())
            }
            Expr::Err(inner, _) => {
                let _ = self.check_expr(inner);
                CheckedType::Named("Result".into())
            }
            Expr::Struct(name, fields, span) => {
                let struct_fields = self.types.get(&name.name).cloned();
                match struct_fields {
                    Some(expected_fields) => {
                        for (fname, fval) in fields {
                            let val_ty = self.check_expr(fval);
                            if let Some(expected) = expected_fields.get(&fname.name) {
                                if !self.types_compatible(&val_ty, expected) && val_ty != CheckedType::Error {
                                    self.error(
                                        format!("field '{}' type mismatch: expected {}, found {}",
                                            fname.name, expected.name(), val_ty.name()),
                                        *span,
                                    );
                                }
                            } else {
                                self.error(
                                    format!("type '{}' has no field '{}'", name.name, fname.name),
                                    *span,
                                );
                            }
                        }
                    }
                    None => {
                        self.error(format!("unknown type '{}'", name.name), *span);
                    }
                }
                CheckedType::Named(name.name.clone())
            }
            Expr::Array(items, _) => {
                if items.is_empty() {
                    CheckedType::Named("Vec".into())
                } else {
                    let first_ty = self.check_expr(&items[0]);
                    for item in &items[1..] {
                        let item_ty = self.check_expr(item);
                        if !self.types_compatible(&first_ty, &item_ty) && item_ty != CheckedType::Error {
                            // soft error — arrays should be homogeneous
                        }
                    }
                    CheckedType::Named("Vec".into())
                }
            }
            Expr::Closure(_, _, _) => CheckedType::Named("fn".into()),
            Expr::PipeClosure(_, body, _) => {
                let _ = self.check_expr(body);
                CheckedType::Named("fn".into())
            }
            Expr::Await(inner, _) => self.check_expr(inner),
            Expr::Comptime(inner, _) => self.check_expr(inner),
        }
    }

    fn types_compatible(&self, found: &CheckedType, expected: &CheckedType) -> bool {
        if found == &CheckedType::Error || expected == &CheckedType::Error {
            return true; // Don't cascade errors
        }
        if found == expected {
            return true;
        }
        // Named types are compatible if they have the same name
        match (found, expected) {
            (CheckedType::Named(a), CheckedType::Named(b)) => a == b,
            // Numeric promotions
            (CheckedType::Int, CheckedType::Float64) => true,
            (CheckedType::Float64, CheckedType::Int) => true,
            (CheckedType::Float32, CheckedType::Float64) => true,
            // Unit compatibility
            (_, CheckedType::Unit) => true,
            _ => false,
        }
    }
}

impl Default for Checker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axiom_lexer::Lexer;
    use axiom_parser::Parser;

    fn check(source: &str) -> Result<(), Vec<CheckError>> {
        let tokens = Lexer::new(source).tokenize();
        let program = Parser::new(tokens).parse_program();
        match program {
            Ok(p) => Checker::new().check_program(&p),
            Err(e) => Err(vec![CheckError {
                message: format!("parse error: {e}"),
                span: e.span,
            }]),
        }
    }

    #[test]
    fn test_simple_addition() {
        let result = check("fn add(a: Int, b: Int) -> Int { return a + b; }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_return_type_mismatch() {
        let result = check("fn bad() -> Int { return true; }");
        assert!(result.is_err());
    }

    #[test]
    fn test_if_condition_bool() {
        let result = check("fn test(x: Int) -> Int { if x > 0 { return 1; } else { return 0; } }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_if_condition_not_bool() {
        let result = check("fn bad(x: Int) -> Int { if x { return 1; } else { return 0; } }");
        assert!(result.is_err());
    }

    #[test]
    fn test_undefined_variable() {
        let result = check("fn bad() -> Int { return x; }");
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_field_access() {
        let result = check("type Point = { x: Float64; y: Float64; } fn get_x(p: Point) -> Float64 { return p.x; }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_unknown_field() {
        let result = check("type Point = { x: Float64; y: Float64; } fn bad(p: Point) -> Float64 { return p.z; }");
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_literal() {
        let result = check("type Point = { x: Float64; y: Float64; } fn make_point() -> Point { return Point{ x: 1.0, y: 2.0 }; }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_let_inference() {
        let result = check("fn test() -> Int { let x = 42; return x; }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_functions() {
        let result = check("fn square(x: Int) -> Int { return x * x; } fn sum_squares(a: Int, b: Int) -> Int { return square(a) + square(b); }");
        assert!(result.is_ok(), "{:?}", result.err());
    }
}
