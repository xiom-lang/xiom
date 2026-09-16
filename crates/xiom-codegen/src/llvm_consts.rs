// XIOM CodeGen -- LLVM Type Constants
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// M14.7: Canonical LLVM type strings used throughout the code generator.
// Replacing ~450 inline string literals with named constants enables:
// - Single place to change LLVM conventions
// - Compiler-checked consistency (typo in a string literal is silently wrong)
// - Easier LLVM version upgrades (e.g. opaque pointers)
//
// Usage: `use crate::llvm_consts::*;` then `format!("  store {LLVM_I64} %v, {LLVM_I64}* %p")`

// === Integer types ===
pub const LLVM_I1: &str = "i1";
pub const LLVM_I8: &str = "i8";
pub const LLVM_I32: &str = "i32";
pub const LLVM_I64: &str = "i64";

// === Float types ===
pub const LLVM_DOUBLE: &str = "double";

// === Void ===
pub const LLVM_VOID: &str = "void";

// === Pointer types ===
/// `i8*` -- generic byte pointer (also used for XIOM `Str` at the ABI)
pub const LLVM_STR_PTR: &str = "i8*";

// === Struct prefix ===
/// `%struct.` prefix used when emitting named LLVM struct types
pub const LLVM_STRUCT: &str = "%struct.";
