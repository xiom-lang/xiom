// XIOM Check — Type Compatibility Rules
// Sprint 6B.3: extracted from lib.rs (was impl Checker method)
use std::collections::HashMap;
use crate::types::CheckedType;

/// Determine if `found` type is compatible with `expected` type.
/// Rules ordered from most-specific to least-specific. First match wins.
/// `interfaces`: map of interface_name -> [(method_name, param_types, return_type)]
pub(crate) fn types_compatible(
    found: &CheckedType,
    expected: &CheckedType,
    interfaces: &HashMap<String, Vec<(String, Vec<String>, Option<String>)>>,
) -> bool {
    // v0.55: Never type (!) is the bottom type — compatible with everything.
    // Functions returning ! never return; match arms with ! bodies are exhaustive.
    if matches!(found, CheckedType::Never) || matches!(expected, CheckedType::Never) {
        return true;
    }
    // Wildcard `_` is compatible with everything
    if matches!(found, CheckedType::Named(n) if n == "_") ||
       matches!(expected, CheckedType::Named(n) if n == "_") {
        return true;
    }
    // Normalize Named("Bool") <-> Bool, Named("Int") <-> Int, etc.
    let found = from_str(found);
    let expected = from_str(expected);

    // Same type: always compatible
    if found == expected {
        return true;
    }
    // Error types are compatible with anything (error recovery)
    if matches!(found, CheckedType::Error) || matches!(expected, CheckedType::Error) {
        return true;
    }
    // Generic type parameters (single uppercase letter) are compatible with any type
    let is_generic_param = |ty: &CheckedType| -> bool {
        if let CheckedType::Named(s) = ty {
            s.len() == 1 && s.chars().next().map_or(false, |c| c.is_ascii_uppercase())
        } else {
            false
        }
    };
    if is_generic_param(&found) || is_generic_param(&expected) {
        return true;
    }
    match (&found, &expected) {
        (CheckedType::Named(a), CheckedType::Named(b)) if a == b => true,
        // Tuple types are broadly compatible
        (CheckedType::Named(n), _) if n.starts_with("Tuple") => true,
        (_, CheckedType::Named(n)) if n.starts_with("Tuple") => true,
        // Integer literals (always `Int`) compatible with any integer-like target
        (CheckedType::Int, other) | (other, CheckedType::Int)
            if other.is_numeric() => true,
        // Self is an alias for the concrete type
        (CheckedType::Named(a), CheckedType::Named(b)) if a == "Self" || b == "Self" => true,
        // Interface/trait names compatible with implementors
        (CheckedType::Named(a), CheckedType::Named(b))
            if interfaces.contains_key(a) || interfaces.contains_key(b) => true,
        // Array/Slice/Vec share the same runtime layout
        (CheckedType::Named(a), CheckedType::Named(b))
            if (a.starts_with("Array") || a.starts_with("Slice") || a.starts_with("Vec")) &&
               (b.starts_with("Array") || b.starts_with("Slice") || b.starts_with("Vec")) &&
               a != b => true,
        // 6A.1: Different named types are NOT compatible
        (CheckedType::Named(_), CheckedType::Named(_)) => false,
        // Wildcard placeholder
        (CheckedType::Named(n), _) if n == "_" => true,
        (_, CheckedType::Named(n)) if n == "_" => true,
        // fn-type compatibility
        (CheckedType::Named(n), CheckedType::Fn(..)) if n == "fn" => true,
        (CheckedType::Fn(..), CheckedType::Named(n)) if n == "fn" => true,
        // Numeric promotions
        (CheckedType::Int, CheckedType::Float64) => true,
        (CheckedType::Float64, CheckedType::Int) => true,
        (CheckedType::Float32, CheckedType::Float64) => true,
        (CheckedType::Float64, CheckedType::Float32) => true,
        (CheckedType::Int, CheckedType::Char) => true,
        (CheckedType::Char, CheckedType::Int) => true,
        // Integer width promotions for FFI compatibility
        (CheckedType::Int, CheckedType::Int32) | (CheckedType::Int32, CheckedType::Int) => true,
        (CheckedType::Int, CheckedType::Int16) | (CheckedType::Int16, CheckedType::Int) => true,
        (CheckedType::Int, CheckedType::Int8)  | (CheckedType::Int8,  CheckedType::Int) => true,
        (CheckedType::Int32, CheckedType::Int16) | (CheckedType::Int16, CheckedType::Int32) => true,
        (CheckedType::Int32, CheckedType::Int8)  | (CheckedType::Int8,  CheckedType::Int32) => true,
        (CheckedType::Int16, CheckedType::Int8)  | (CheckedType::Int8,  CheckedType::Int16) => true,
        // Unsigned integer compatibility
        (CheckedType::UInt, CheckedType::UInt32) | (CheckedType::UInt32, CheckedType::UInt) => true,
        (CheckedType::UInt, CheckedType::UInt16) | (CheckedType::UInt16, CheckedType::UInt) => true,
        (CheckedType::UInt, CheckedType::UInt8)  | (CheckedType::UInt8,  CheckedType::UInt) => true,
        // Signed↔unsigned (FFI common)
        (CheckedType::Int,    CheckedType::UInt32) | (CheckedType::UInt32, CheckedType::Int) => true,
        (CheckedType::Int32,  CheckedType::UInt32) | (CheckedType::UInt32, CheckedType::Int32) => true,
        (CheckedType::Int,    CheckedType::UInt)   | (CheckedType::UInt,   CheckedType::Int) => true,
        // Unit compatibility
        (_, CheckedType::Unit) => true,
        _ => false,
    }
}

/// Normalize "Bool" (Named) → Bool (checked type variant).
fn from_str(ty: &CheckedType) -> CheckedType {
    match ty {
        CheckedType::Named(n) => match n.as_str() {
            "Bool" => CheckedType::Bool,
            "Int" => CheckedType::Int,
            "Int8" => CheckedType::Int8,
            "Int16" => CheckedType::Int16,
            "Int32" => CheckedType::Int32,
            "Int64" => CheckedType::Int,
            "UInt" => CheckedType::UInt,
            "UInt8" => CheckedType::UInt8,
            "UInt16" => CheckedType::UInt16,
            "UInt32" => CheckedType::UInt32,
            "Float32" => CheckedType::Float32,
            "Float64" => CheckedType::Float64,
            "Str" => CheckedType::Str,
            "Char" => CheckedType::Char,
            "Unit" => CheckedType::Unit,
            _ => ty.clone(),
        },
        _ => ty.clone(),
    }
}
