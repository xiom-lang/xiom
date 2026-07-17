// XIOM — Checker Types
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use xiom_ast::*;
use std::fmt;

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
    /// Function pointer type: fn(T, U) -> V
    Fn(Vec<CheckedType>, Box<CheckedType>),
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
            Type::Fn(params, ret) => CheckedType::Fn(
                params.iter().map(CheckedType::from_ast_type).collect(),
                Box::new(CheckedType::from_ast_type(ret)),
            ),
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
            "_" => CheckedType::Int, // wildcard placeholder
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
            CheckedType::Fn(params, ret) => {
                let params_str: Vec<String> = params.iter().map(|p| p.name()).collect();
                format!("fn({}) -> {}", params_str.join(", "), ret.name())
            }
            CheckedType::Generic(s) => s.clone(),
            CheckedType::Error => "<error>".into(),
        }
    }

    /// Convert CheckedType back to an AST Type for codegen consumption.
    pub fn to_ast_type(&self) -> Type {
        match self {
            CheckedType::Bool => Type::Named(Ident::new("Bool", Span::new(0, 0)), vec![]),
            CheckedType::Int => Type::Named(Ident::new("Int", Span::new(0, 0)), vec![]),
            CheckedType::Int8 => Type::Named(Ident::new("Int8", Span::new(0, 0)), vec![]),
            CheckedType::Int16 => Type::Named(Ident::new("Int16", Span::new(0, 0)), vec![]),
            CheckedType::Int32 => Type::Named(Ident::new("Int32", Span::new(0, 0)), vec![]),
            CheckedType::Int64 => Type::Named(Ident::new("Int64", Span::new(0, 0)), vec![]),
            CheckedType::UInt => Type::Named(Ident::new("UInt", Span::new(0, 0)), vec![]),
            CheckedType::UInt8 => Type::Named(Ident::new("UInt8", Span::new(0, 0)), vec![]),
            CheckedType::UInt16 => Type::Named(Ident::new("UInt16", Span::new(0, 0)), vec![]),
            CheckedType::UInt32 => Type::Named(Ident::new("UInt32", Span::new(0, 0)), vec![]),
            CheckedType::UInt64 => Type::Named(Ident::new("UInt64", Span::new(0, 0)), vec![]),
            CheckedType::Float32 => Type::Named(Ident::new("Float32", Span::new(0, 0)), vec![]),
            CheckedType::Float64 => Type::Named(Ident::new("Float64", Span::new(0, 0)), vec![]),
            CheckedType::Char => Type::Named(Ident::new("Char", Span::new(0, 0)), vec![]),
            CheckedType::Str => Type::Named(Ident::new("Str", Span::new(0, 0)), vec![]),
            CheckedType::Unit => Type::Named(Ident::new("()", Span::new(0, 0)), vec![]),
            CheckedType::Never => Type::Named(Ident::new("!", Span::new(0, 0)), vec![]),
            CheckedType::Named(s) => Type::Named(Ident::new(s.clone(), Span::new(0, 0)), vec![]),
            CheckedType::Fn(params, ret) => Type::Fn(
                params.iter().map(|p| p.to_ast_type()).collect(),
                Box::new(ret.to_ast_type()),
            ),
            CheckedType::Generic(s) => Type::Named(Ident::new(s.clone(), Span::new(0, 0)), vec![]),
            CheckedType::Error => Type::Named(Ident::new("<error>", Span::new(0, 0)), vec![]),
        }
    }
}

// ============================================================================
// Function signature
// ============================================================================

#[derive(Debug, Clone)]
pub struct FnSig {
    pub params: Vec<(String, CheckedType)>,
    pub return_type: Option<CheckedType>,
    pub generics: Vec<String>,
    /// True when the function body references `this` (implicit receiver),
    /// meaning the first arg in a static call is the explicit receiver.
    pub uses_implicit_this: bool,
}

// ============================================================================
// Check error
// ============================================================================

#[derive(Debug, Clone)]
pub struct CheckError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type error at {}: {}", self.span, self.message)
    }
}
