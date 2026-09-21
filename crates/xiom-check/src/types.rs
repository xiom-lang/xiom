// XIOM -- Checker Types
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Type definitions for the XIOM type checker.
//!
//! This module defines the canonical type representation used throughout the
//! checker: [`CheckedType`] for type values, [`FnSig`] for function signatures,
//! [`CheckError`] for error reporting, and the [`TypeArena`] for type interning.

use xiom_ast::*;
use std::collections::HashMap;
use std::fmt;
use std::sync::{OnceLock, RwLock};

// ============================================================================
// Type Interning (Stage 2c: process-global interned type identity)
// ============================================================================

/// Opaque index into the process-global type intern table. Two `TypeId`s are
/// equal iff their CANONICAL type names are structurally equal, so equality
/// is O(1) and names carry no spelling variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeId(u32);

/// Intern table for canonical type names.
///
/// Process-global and append-only: `TypeId`s are stable for the lifetime of
/// the process and resolve from any checker, LSP snapshot or test. The table
/// is stored once behind an `RwLock`; interned names are leaked (`&'static
/// str`) so rendering never holds the lock. `contains_param` is derived
/// structurally (see [`crate::structural`]).
///
/// A handle is a zero-sized value: `TypeArena::new()` and
/// `TypeArena::global()` both refer to the one table, which is what makes
/// `CheckedType::Named(TypeId)` comparable across checkers.
#[derive(Debug, Clone, Copy, Default)]
pub struct TypeArena;

#[derive(Default)]
struct Interner {
    /// Canonical name -> id.
    by_name: HashMap<&'static str, TypeId>,
    /// id -> canonical name (index = TypeId.0).
    names: Vec<&'static str>,
    /// id -> whether the type contains a generic parameter anywhere.
    contains_param: Vec<bool>,
}

fn interner() -> &'static RwLock<Interner> {
    static INTERNER: OnceLock<RwLock<Interner>> = OnceLock::new();
    INTERNER.get_or_init(|| RwLock::new(Interner::default()))
}

/// Lock recovery: an interner panic cannot leave a half-written entry (every
/// mutation is a push), so poisoning is recoverable state, not a fault.
fn write_lock() -> std::sync::RwLockWriteGuard<'static, Interner> {
    interner().write().unwrap_or_else(|e| e.into_inner())
}

fn read_lock() -> std::sync::RwLockReadGuard<'static, Interner> {
    interner().read().unwrap_or_else(|e| e.into_inner())
}

impl TypeId {
    /// Canonical name of this interned type.
    pub fn name(self) -> &'static str {
        TypeArena.name_of(self)
    }

    /// Process-stable numeric id (for debug output only; do not persist).
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

/// Canonical structural interning as a conversion: `Named(s.into())` and
/// `CheckedType::Named(s.into())` intern through the one entry point, so a
/// `TypeId` can never be built from a non-canonical spelling.
impl From<&str> for TypeId {
    fn from(name: &str) -> Self {
        TypeArena::new().intern_type_name(name)
    }
}

impl From<String> for TypeId {
    fn from(name: String) -> Self {
        TypeArena::new().intern_type_name(&name)
    }
}

/// An interned type id's semantic content IS its canonical name. The
/// `Display`/`PartialEq<str>`/`Deref<Target = str>` surface is the standard
/// interned-symbol API (cf. rustc_span::Symbol): pattern matches and
/// diagnostics can treat the id as the name without un-interning boilerplate.
impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl PartialEq<str> for TypeId {
    fn eq(&self, other: &str) -> bool {
        self.name() == other
    }
}

impl PartialEq<&str> for TypeId {
    fn eq(&self, other: &&str) -> bool {
        self.name() == *other
    }
}

impl PartialEq<TypeId> for str {
    fn eq(&self, other: &TypeId) -> bool {
        other.name() == self
    }
}

impl std::ops::Deref for TypeId {
    type Target = str;
    fn deref(&self) -> &str {
        self.name()
    }
}

impl TypeArena {
    /// Global table handle. Historical name kept for call sites; there is
    /// exactly one type universe per process.
    pub fn new() -> Self {
        TypeArena
    }

    /// Global table handle.
    pub fn global() -> &'static TypeArena {
        static GLOBAL: OnceLock<TypeArena> = OnceLock::new();
        GLOBAL.get_or_init(TypeArena::new)
    }

    /// Intern a type name structurally. Whitespace/format variants of the
    /// same structure share one `TypeId`; `contains_param` is derived from
    /// the parsed shape. This is the ONLY interning entry point -- raw
    /// interning would break `TypeId` equality guarantees.
    pub fn intern_type_name(&self, name: &str) -> TypeId {
        let canonical = crate::structural::canonical_type_name(name);
        let contains = type_shape_contains_param(&crate::structural::parse_type_shape(&canonical));
        let mut table = write_lock();
        if let Some(id) = table.by_name.get(canonical.as_str()) {
            return *id;
        }
        // Leak the canonical spelling: the table is append-only, so entries
        // live for the process and can be handed out without the lock.
        let leaked: &'static str = Box::leak(canonical.into_boxed_str());
        let id = TypeId(table.names.len() as u32);
        table.names.push(leaked);
        table.contains_param.push(contains);
        table.by_name.insert(leaked, id);
        id
    }

    /// Canonical name for a `TypeId`.
    pub fn name_of(&self, id: TypeId) -> &'static str {
        read_lock().names[id.0 as usize]
    }

    /// Canonical spelling of a type name (structural identity form).
    pub fn canonical_name(name: &str) -> String {
        crate::structural::canonical_type_name(name)
    }

    /// True when the type (or any nested type) contains a generic parameter.
    /// Monomorphisation skips `contains_param` == false in O(1).
    pub fn contains_param(&self, id: TypeId) -> bool {
        read_lock().contains_param[id.0 as usize]
    }
}

/// True when a parsed shape contains a generic parameter anywhere.
pub fn type_shape_contains_param(shape: &crate::structural::TypeShape) -> bool {
    use crate::structural::TypeShape;
    match shape {
        TypeShape::Generic(_) => true,
        TypeShape::Pointer(inner) => type_shape_contains_param(inner),
        TypeShape::Named { args, .. } => args.iter().any(type_shape_contains_param),
        _ => false,
    }
}

// ============================================================================
// Type representation for the checker
// ============================================================================

#[derive(Clone, PartialEq)]
/// The canonical type representation in the XIOM type system.
///
/// Covers all XIOM types: primitives (`Int`, `Bool`, `Str`, ...), compound types
/// (`Vec`, `Map`, `Option`, `Result`, ...), references (`&T`, `&mut T`), raw pointers
/// (`*T`), user-defined types (`Named` with optional generics), function types,
/// interface types, generic parameters, and `impl Trait` opaque types.
///
/// Notable variants:
/// - [`CheckedType::Named`] -- user-defined/compound types by INTERNED
///   canonical name ([`TypeId`]); equality is structural and O(1)
/// - [`CheckedType::Error`] -- poison type used after a type error to suppress cascading errors
/// - [`CheckedType::Wildcard`] -- the `_` type, compatible with anything
/// - [`CheckedType::ImplTrait`] -- opaque existential return type
pub enum CheckedType {
    Bool,
    Int, Int8, Int16, Int32, Int64, Int128,
    UInt, UInt8, UInt16, UInt32, UInt64, UInt128,
    Float32, Float64, Float128,
    Char,
    Str,
    Unit,
    Never,
    /// A user-defined or compound type by interned canonical name.
    Named(TypeId),
    /// A generic type parameter (still unresolved)
    Generic(String),
    /// Function pointer type: fn(T, U) -> V
    Fn(Vec<CheckedType>, Box<CheckedType>),
    /// Error type -- used when type checking fails
    Error,
    /// Opaque impl Trait return type (M9.6)
    ImplTrait(Vec<String>),
}

/// Renders `Named` as `Named("Vec[Int]")` (the derived pre-Stage-2c shape):
/// diagnostics and traces must not leak raw `TypeId` numbers.
impl fmt::Debug for CheckedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckedType::Bool => write!(f, "Bool"),
            CheckedType::Int => write!(f, "Int"),
            CheckedType::Int8 => write!(f, "Int8"),
            CheckedType::Int16 => write!(f, "Int16"),
            CheckedType::Int32 => write!(f, "Int32"),
            CheckedType::Int64 => write!(f, "Int64"),
            CheckedType::Int128 => write!(f, "Int128"),
            CheckedType::UInt => write!(f, "UInt"),
            CheckedType::UInt8 => write!(f, "UInt8"),
            CheckedType::UInt16 => write!(f, "UInt16"),
            CheckedType::UInt32 => write!(f, "UInt32"),
            CheckedType::UInt64 => write!(f, "UInt64"),
            CheckedType::UInt128 => write!(f, "UInt128"),
            CheckedType::Float32 => write!(f, "Float32"),
            CheckedType::Float64 => write!(f, "Float64"),
            CheckedType::Float128 => write!(f, "Float128"),
            CheckedType::Char => write!(f, "Char"),
            CheckedType::Str => write!(f, "Str"),
            CheckedType::Unit => write!(f, "Unit"),
            CheckedType::Never => write!(f, "Never"),
            CheckedType::Named(id) => write!(f, "Named({:?})", id.name()),
            CheckedType::Generic(name) => write!(f, "Generic({name:?})"),
            CheckedType::Fn(params, ret) => {
                f.debug_tuple("Fn").field(params).field(ret).finish()
            }
            CheckedType::Error => write!(f, "Error"),
            CheckedType::ImplTrait(traits) => {
                f.debug_tuple("ImplTrait").field(traits).finish()
            }
        }
    }
}

impl CheckedType {
    /// Construct a `Named` type from a name, interning its canonical form.
    /// This is the single construction point for named/compound types:
    /// `TypeId` equality is structural identity.
    pub fn named(name: impl AsRef<str>) -> Self {
        CheckedType::Named(TypeArena::new().intern_type_name(name.as_ref()))
    }

    /// Convert from AST Type to checked type representation
    pub fn from_ast_type(ty: &Type) -> Self {
        match ty {
            Type::Named(ident, _) => Self::from_str(&ident.name),
            Type::Ref(inner) => CheckedType::from_ast_type(inner),
            Type::MutRef(inner) => CheckedType::from_ast_type(inner),
            // BUG 51 (2026-08-18): PRESERVE container type args so match
            // payload bindings get the inner type ("Option[MyRc]" -> Some(up)
            // binds up: MyRc) -- the erased "Option" forced payload bindings to
            // the `_` wildcard, and method calls on them fell to the sorted
            // wildcard lookup (Option.get before MyRc.get -> "cannot compare
            // Option with Int").
            Type::Option(inner) => CheckedType::named(format!("Option[{}]", CheckedType::from_ast_type(inner).name())),
            Type::Result(ok, err) => CheckedType::named(format!(
                "Result[{}, {}]",
                CheckedType::from_ast_type(ok).name(),
                CheckedType::from_ast_type(err).name()
            )),
            Type::Vec(inner) => CheckedType::named(format!("Vec[{}]", CheckedType::from_ast_type(inner).name())),
            Type::Map(k, v) => CheckedType::named(format!(
                "Map[{}, {}]",
                CheckedType::from_ast_type(k).name(),
                CheckedType::from_ast_type(v).name()
            )),
            Type::Set(inner) => CheckedType::named(format!("Set[{}]", CheckedType::from_ast_type(inner).name())),
            Type::Slice(inner) => CheckedType::named(format!("Slice[{}]", CheckedType::from_ast_type(inner).name())),
            Type::Tuple(types) => {
                // M20: Include element types in tuple name to avoid collisions
                // (Int, Str) -> Tuple__Int__Str, not just Tuple2
                let elem_names: Vec<String> = types.iter()
                    .map(|t| CheckedType::from_ast_type(t).name())
                    .collect();
                CheckedType::named(format!("Tuple__{}", elem_names.join("__")))
            }
            Type::Ptr(inner) => {
                // Encode *T as "*Tname" to preserve pointee type for deref resolution.
                // Previously this was always "Ptr", losing the target struct type.
                let inner_name = CheckedType::from_ast_type(inner).name();
                CheckedType::named(format!("*{}", inner_name))
            },
            // smoke_array_zip fix (2026-09-11): keep the ELEMENT type so
            // `zipped[0]` can resolve the tuple and `.0` type-checks.
            // Previously every array erased to "Array" and indexing yielded Int.
            Type::Array(_, elem) => CheckedType::named(format!(
                "Array[{}]",
                CheckedType::from_ast_type(elem).name()
            )),
            Type::Fn(params, ret) => CheckedType::Fn(
                params.iter().map(CheckedType::from_ast_type).collect(),
                Box::new(CheckedType::from_ast_type(ret)),
            ),
            Type::ImplTrait(traits) => CheckedType::ImplTrait(
                traits.iter().map(|t| t.name.clone()).collect(),
            ),
            Type::AnonStruct(fields) => {
                let parts: Vec<String> = fields.iter()
                    .map(|f| format!("{}_{}", f.name.name, CheckedType::from_ast_type(&f.ty).name()))
                    .collect();
                CheckedType::named(format!("_Anon__{}", parts.join("__")))
            },
            Type::Never => CheckedType::Never,
        }
    }

    pub fn from_str(s: &str) -> Self {
        // Stage 2c: every Named value is interned CANONICALLY. Spelling
        // variants ("Result[Int,Str]" / " Result[ Int , Str ] ") therefore
        // produce the same TypeId, and primitives with stray whitespace
        // classify as primitives instead of as named lookalikes.
        let canonical = crate::structural::canonical_type_name(s);
        match canonical.as_str() {
            "Bool" => CheckedType::Bool,
            "Int" => CheckedType::Int,
            "Int8" => CheckedType::Int8,
            "Int16" => CheckedType::Int16,
            "Int32" => CheckedType::Int32,
            "Int64" => CheckedType::Int64,
            "Int128" => CheckedType::Int128,
            "UInt" => CheckedType::UInt,
            "UInt8" => CheckedType::UInt8,
            "UInt16" => CheckedType::UInt16,
            "UInt32" => CheckedType::UInt32,
            "UInt64" => CheckedType::UInt64,
            "UInt128" => CheckedType::UInt128,
            "Float32" => CheckedType::Float32,
            "Float64" => CheckedType::Float64,
            "Float128" => CheckedType::Float128,
            "Char" => CheckedType::Char,
            "Str" => CheckedType::Str,
            "()" => CheckedType::Unit,
            "_" => CheckedType::Int, // wildcard placeholder
            _ => CheckedType::Named(TypeArena::new().intern_type_name(&canonical)),
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self,
            CheckedType::Int | CheckedType::Int8 | CheckedType::Int16 |
            CheckedType::Int32 | CheckedType::Int64 | CheckedType::Int128 |
            CheckedType::UInt | CheckedType::UInt8 | CheckedType::UInt16 |
            CheckedType::UInt32 | CheckedType::UInt64 | CheckedType::UInt128 |
            CheckedType::Float32 | CheckedType::Float64 | CheckedType::Float128
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(self,
            CheckedType::Int | CheckedType::Int8 | CheckedType::Int16 |
            CheckedType::Int32 | CheckedType::Int64 |
            CheckedType::UInt | CheckedType::UInt8 | CheckedType::UInt16 |
            CheckedType::UInt32 | CheckedType::UInt64 | CheckedType::UInt128
        )
    }

    /// Return true if this is a pointer-like type: the generic `Ptr` or
    /// a specific `*T` encoded as `"*Tname"`.
    pub fn as_ptr_like(&self) -> bool {
        matches!(self, CheckedType::Named(id) if id.name() == "Ptr" || id.name().starts_with('*'))
    }

    pub fn name(&self) -> String {
        match self {
            CheckedType::Bool => "Bool".into(),
            CheckedType::Int => "Int".into(),
            CheckedType::Int8 => "Int8".into(),
            CheckedType::Int16 => "Int16".into(),
            CheckedType::Int32 => "Int32".into(),
            CheckedType::Int64 => "Int64".into(),
            CheckedType::Int128 => "Int128".into(),
            CheckedType::UInt => "UInt".into(),
            CheckedType::UInt8 => "UInt8".into(),
            CheckedType::UInt16 => "UInt16".into(),
            CheckedType::UInt32 => "UInt32".into(),
            CheckedType::UInt64 => "UInt64".into(),
            CheckedType::UInt128 => "UInt128".into(),
            CheckedType::Float32 => "Float32".into(),
            CheckedType::Float64 => "Float64".into(),
            CheckedType::Float128 => "Float128".into(),
            CheckedType::Char => "Char".into(),
            CheckedType::Str => "Str".into(),
            CheckedType::Unit => "()".into(),
            CheckedType::Never => "!".into(),
            CheckedType::Named(id) => id.name().to_string(),
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
            CheckedType::Int128 => Type::Named(Ident::new("Int128", Span::new(0, 0)), vec![]),
            CheckedType::UInt => Type::Named(Ident::new("UInt", Span::new(0, 0)), vec![]),
            CheckedType::UInt8 => Type::Named(Ident::new("UInt8", Span::new(0, 0)), vec![]),
            CheckedType::UInt16 => Type::Named(Ident::new("UInt16", Span::new(0, 0)), vec![]),
            CheckedType::UInt32 => Type::Named(Ident::new("UInt32", Span::new(0, 0)), vec![]),
            CheckedType::UInt64 => Type::Named(Ident::new("UInt64", Span::new(0, 0)), vec![]),
            CheckedType::UInt128 => Type::Named(Ident::new("UInt128", Span::new(0, 0)), vec![]),
            CheckedType::Float32 => Type::Named(Ident::new("Float32", Span::new(0, 0)), vec![]),
            CheckedType::Float64 => Type::Named(Ident::new("Float64", Span::new(0, 0)), vec![]),
            CheckedType::Float128 => Type::Named(Ident::new("Float128", Span::new(0, 0)), vec![]),
            CheckedType::Char => Type::Named(Ident::new("Char", Span::new(0, 0)), vec![]),
            CheckedType::Str => Type::Named(Ident::new("Str", Span::new(0, 0)), vec![]),
            CheckedType::Unit => Type::Named(Ident::new("()", Span::new(0, 0)), vec![]),
            CheckedType::Never => Type::Named(Ident::new("!", Span::new(0, 0)), vec![]),
            CheckedType::Named(id) => Type::Named(Ident::new(id.name().to_string(), Span::new(0, 0)), vec![]),
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
// TypeCause -- error provenance (5c-R: 8 reason codes, rustc ObligationCause)
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
    /// AUDIT FIX (readiness Stage 2b): the ErrorGuaranteed PROOF TOKEN --
    /// real now. Downstream passes can rely on its presence to skip
    /// error-poisoned work instead of string-matching messages.
    pub guaranteed: xiom_ast::ErrorGuaranteed,
}

/// Stage 3 Item A: the full-catalog body-check report returned by
/// [`crate::Checker::check_catalog_corpus`].
///
/// `findings` is the gate for flipping catalog-body warnings into hard
/// errors: the flip lands only when this list is EMPTY (and `errors` is
/// empty, i.e. every module import resolved).
#[derive(Debug, Clone, Default)]
pub struct CatalogCorpusReport {
    /// Catalog-body findings (`message` starts with "catalog body").
    pub findings: Vec<CheckError>,
    /// Other warnings collected while checking the corpus.
    pub warnings: Vec<CheckError>,
    /// Hard errors (unresolved imports, parse failures, ...).
    pub errors: Vec<CheckError>,
    /// D1: recoverable PARSE diagnostics from catalog files. These are hard
    /// errors for the gate: the parser recovered by dropping declarations, so
    /// the module's surface is incomplete (downstream undefined-name findings
    /// are cascades of this).
    pub parse_errors: Vec<CheckError>,
}

impl CatalogCorpusReport {
    /// True when the corpus is clean: no catalog findings, no hard errors and
    /// no catalog parse diagnostics.
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty() && self.errors.is_empty() && self.parse_errors.is_empty()
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}: {}", self.cause, self.span, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interning_canonicalizes_structural_variants() {
        let arena = TypeArena::new();
        let a = arena.intern_type_name("Result[Int,Str]");
        let b = arena.intern_type_name("Result[Int, Str]");
        let c = arena.intern_type_name(" Result[ Int , Str ] ");
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(arena.name_of(a), "Result[Int, Str]");
    }

    #[test]
    fn interning_tracks_generic_params_structurally() {
        let arena = TypeArena::new();
        let generic = arena.intern_type_name("Vec[Option[T]]");
        assert!(arena.contains_param(generic));
        let concrete = arena.intern_type_name("Vec[Option[Int]]");
        assert!(!arena.contains_param(concrete));
        assert_ne!(generic, concrete);
    }

    #[test]
    fn canonical_name_normalizes_spacing_only() {
        assert_eq!(TypeArena::canonical_name("Map[Str,Vec[Int]]"), "Map[Str, Vec[Int]]");
        assert_eq!(TypeArena::canonical_name("Int"), "Int");
        assert_eq!(TypeArena::canonical_name("*T"), "*T");
    }

    #[test]
    fn checked_named_equality_is_structural() {
        // Stage 2c: spelling variants produce EQUAL values and the Debug
        // rendering stays human-readable (never leaks TypeId numbers).
        let a = CheckedType::named("Map[Str,Vec[Int]]");
        let b = CheckedType::named(" Map[ Str , Vec [ Int ] ] ");
        assert_eq!(a, b);
        assert_eq!(a.name(), "Map[Str, Vec[Int]]");
        assert_eq!(format!("{a:?}"), "Named(\"Map[Str, Vec[Int]]\")");
        // Primitive classification ignores stray whitespace.
        assert_eq!(CheckedType::from_str(" Int "), CheckedType::Int);
    }
}
