// XIOM — Checker Types
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! Type definitions for the XIOM type checker.
//!
//! This module defines the canonical type representation used throughout the
//! checker: [`CheckedType`] for type values, [`FnSig`] for function signatures,
//! [`CheckError`] for error reporting, and the [`TypeArena`] for type interning.

use xiom_ast::*;
use std::fmt;

// ============================================================================
// Type Interning (5c-R: TypeId + arena — rustc lesson from TyCtxt)
// ============================================================================

/// Opaque index into the type arena. O(1) equality, zero heap indirection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(u32);

/// Arena that interns type names and tracks the `CONTAINS_PARAM` flag.
/// Named types resolve to TypeIds for O(1) comparison; the flag enables
/// monomorphisation to skip non-generic types in O(1).
#[derive(Debug, Clone, Default)]
pub struct TypeArena {
    /// Interned type names. Index = TypeId.0.
    names: Vec<String>,
    /// Whether each interned type contains a generic parameter somewhere in
    /// its definition (e.g., `Vec[T]`, `Option[T]`, user-generic `Box[T]`).
    contains_param: Vec<bool>,
}

impl TypeArena {
    pub fn new() -> Self { Self { names: Vec::new(), contains_param: Vec::new() } }

    /// Intern a type name, returning its TypeId. If the name is already
    /// present, the existing ID is returned (deduplication).
    pub fn intern(&mut self, name: &str, contains_param: bool) -> TypeId {
        if let Some(pos) = self.names.iter().position(|n| n == name) {
            return TypeId(pos as u32);
        }
        let id = TypeId(self.names.len() as u32);
        self.names.push(name.to_string());
        self.contains_param.push(contains_param);
        id
    }

    /// Look up the string name for a TypeId.
    pub fn name_of(&self, id: TypeId) -> &str {
        &self.names[id.0 as usize]
    }

    /// True when the type (or any nested type) contains a generic parameter.
    /// Monomorphisation skips `contains_param` == false in O(1).
    pub fn contains_param(&self, id: TypeId) -> bool {
        self.contains_param[id.0 as usize]
    }
}

// ============================================================================
// Type representation for the checker
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
/// The canonical type representation in the XIOM type system.
///
/// Covers all XIOM types: primitives (`Int`, `Bool`, `Str`, ...), compound types
/// (`Vec`, `Map`, `Option`, `Result`, ...), references (`&T`, `&mut T`), raw pointers
/// (`*T`), user-defined types (`Named` with optional generics), function types,
/// interface types, generic parameters, and `impl Trait` opaque types.
///
/// Notable variants:
/// - [`CheckedType::Named`] — user-defined types with optional generic arguments
/// - [`CheckedType::Error`] — poison type used after a type error to suppress cascading errors
/// - [`CheckedType::Wildcard`] — the `_` type, compatible with anything
/// - [`CheckedType::ImplTrait`] — opaque existential return type
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
    /// Opaque impl Trait return type (M9.6)
    ImplTrait(Vec<String>),
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
                // M20: Include element types in tuple name to avoid collisions
                // (Int, Str) → Tuple__Int__Str, not just Tuple2
                let elem_names: Vec<String> = types.iter()
                    .map(|t| CheckedType::from_ast_type(t).name())
                    .collect();
                CheckedType::Named(format!("Tuple__{}", elem_names.join("__")))
            }
            Type::Ptr(inner) => {
                // Encode *T as "*Tname" to preserve pointee type for deref resolution.
                // Previously this was always "Ptr", losing the target struct type.
                let inner_name = CheckedType::from_ast_type(inner).name();
                CheckedType::Named(format!("*{}", inner_name))
            },
            Type::Array(_, _) => CheckedType::Named("Array".into()),
            Type::Fn(params, ret) => CheckedType::Fn(
                params.iter().map(CheckedType::from_ast_type).collect(),
                Box::new(CheckedType::from_ast_type(ret)),
            ),
            Type::ImplTrait(traits) => CheckedType::ImplTrait(
                traits.iter().map(|t| t.name.clone()).collect(),
            ),
            Type::AnonStruct(fields) => {
                // 5c.33: Encode as a synthetic named type using field signature.
                // The checker registers this in the type table when first seen.
                let parts: Vec<String> = fields.iter()
                    .map(|f| format!("{}_{}", f.name.name, CheckedType::from_ast_type(&f.ty).name()))
                    .collect();
                CheckedType::Named(format!("_Anon__{}", parts.join("__")))
            },
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

    /// Return true if this is a pointer-like type: the generic `Ptr` or
    /// a specific `*T` encoded as `"*Tname"`.
    pub fn as_ptr_like(&self) -> bool {
        matches!(self, CheckedType::Named(s) if s == "Ptr" || s.starts_with('*'))
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
            CheckedType::ImplTrait(traits) => format!("impl {}", traits.join(" + ")),
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
            CheckedType::ImplTrait(traits) => {
                let idents: Vec<Ident> = traits.iter()
                    .map(|t| Ident::new(t.clone(), Span::new(0, 0)))
                    .collect();
                Type::ImplTrait(idents)
            }
        }
    }
}

// ============================================================================
// Function signature
// ============================================================================

#[derive(Debug, Clone)]
/// A function signature as resolved by the type checker.
///
/// Contains parameter names with their resolved types, the optional return type,
/// and generic parameters for monomorphisation. Used for call-site validation
/// and code generation.
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

// ============================================================================
// TypeCause — error provenance (5c-R: 8 reason codes, rustc ObligationCause)
// ============================================================================

/// Why a type error occurred. Threaded through every error() call so
/// diagnostics can say "expected X because Y" instead of bare "type mismatch".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeCause {
    ContractRequires,
    ContractEnsures,
    InvariantViolation,
    TypeMismatch,
    UndefinedVariable,
    BadMethodCall,
    BadFieldAccess,
    Other,
}

impl fmt::Display for TypeCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeCause::ContractRequires => write!(f, "contract requires clause violated"),
            TypeCause::ContractEnsures => write!(f, "contract ensures clause not proven"),
            TypeCause::InvariantViolation => write!(f, "invariant broken"),
            TypeCause::TypeMismatch => write!(f, "type mismatch"),
            TypeCause::UndefinedVariable => write!(f, "undefined variable"),
            TypeCause::BadMethodCall => write!(f, "method not found"),
            TypeCause::BadFieldAccess => write!(f, "field not found"),
            TypeCause::Other => write!(f, "type error"),
        }
    }
}

#[derive(Debug, Clone)]
/// A type error produced by the checker.
///
/// Contains a human-readable message, the source span where the error occurred,
/// and a [`TypeCause`] categorisation for cause-aware diagnostics (e.g. "type
/// mismatch", "undefined variable", "cannot call method on non-struct type").
pub struct CheckError {
    pub message: String,
    pub span: Span,
    /// 5c-R: Why this error occurred (enables cause-aware diagnostics)
    pub cause: TypeCause,
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}: {}", self.cause, self.span, self.message)
    }
}
