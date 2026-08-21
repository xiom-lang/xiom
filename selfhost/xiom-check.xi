// XIOM -- Self-Hosted Type Checker
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

// Checks types on AST nodes produced by the parser.
// For bootstrap, operates on hardcoded AST data.
// Type kinds: 0=Unknown, 1=Int, 2=Bool, 3=Float64, 4=Str, 5=Void, 9=Error

module checker {

// ============================================================================
// Type representation
// ============================================================================
pub type CheckedType = {
  kind: Int;
  name: Int;    // index into type name table (not used in bootstrap)
} derive[Eq, Clone]

pub fn int_type() -> CheckedType {
  return CheckedType{ kind: 1, name: 0 };
}

pub fn bool_type() -> CheckedType {
  return CheckedType{ kind: 2, name: 0 };
}

pub fn error_type() -> CheckedType {
  return CheckedType{ kind: 9, name: 0 };
}

pub fn void_type() -> CheckedType {
  return CheckedType{ kind: 5, name: 0 };
}

// ============================================================================
// Type checking functions
// ============================================================================

// Check if two types are compatible
pub fn types_compatible(a: CheckedType, b: CheckedType) -> Bool {
  if a.kind == 9 { return true; }  // Error type compatible with anything
  if b.kind == 9 { return true; }
  if a.kind == b.kind { return true; }
  // Int -> Float64 promotion
  if a.kind == 1 && b.kind == 3 { return true; }
  // Unit compatible with anything
  if b.kind == 5 { return true; }
  return false;
}

// Check a binary expression: left OP right -> result type
pub fn check_binary(left_ty: CheckedType, op: Int, right_ty: CheckedType) -> CheckedType {
  // Comparison ops (>, <, ==, !=, <=, >=) always return Bool
  if op == 57 { return bool_type(); }  // >
  if op == 58 { return bool_type(); }  // <
  if op == 41 { return bool_type(); }  // ==
  if op == 42 { return bool_type(); }  // !=
  if op == 59 { return bool_type(); }  // <=
  if op == 60 { return bool_type(); }  // >=

  // Arithmetic ops (+, -, *, /) require numeric types
  if op == 43 || op == 44 || op == 45 || op == 47 {
    if left_ty.kind != 1 && left_ty.kind != 3 { return error_type(); }
    if right_ty.kind != 1 && right_ty.kind != 3 { return error_type(); }
    // Result type is the wider of the two
    if left_ty.kind == 3 || right_ty.kind == 3 {
      return CheckedType{ kind: 3, name: 0 };
    }
    return int_type();
  }

  // Logical ops (&&, ||) require Bool
  if op == 61 || op == 62 {
    if left_ty.kind != 2 { return error_type(); }
    if right_ty.kind != 2 { return error_type(); }
    return bool_type();
  }

  // Assignment (=)
  if op == 40 {
    return right_ty;
  }

  return error_type();
}

// Check a return statement
pub fn check_return(expr_ty: CheckedType, expected_ret: CheckedType) -> CheckedType {
  if types_compatible(expr_ty, expected_ret) {
    return void_type();
  }
  return error_type();
}

// Check a let/var binding
pub fn check_let(val_ty: CheckedType, annot_ty: CheckedType) -> CheckedType {
  if annot_ty.kind == 0 { return val_ty; }  // No annotation -- infer
  if types_compatible(val_ty, annot_ty) {
    return annot_ty;
  }
  return error_type();
}

// ============================================================================
// Test harness -- verify type checker rules
// ============================================================================

pub fn test_int_compatible() -> Int {
  let a = int_type();
  let b = int_type();
  if !(types_compatible(a, b)) { return 1; }
  return 0;
}

pub fn test_error_compatible() -> Int {
  let a = error_type();
  let b = int_type();
  if !(types_compatible(a, b)) { return 1; }
  return 0;
}

pub fn test_arithmetic_int() -> Int {
  let a = int_type();
  let b = int_type();
  let res = check_binary(a, 43, b); // +
  if res.kind != 1 { return 1; }
  return 0;
}

pub fn test_comparison_bool() -> Int {
  let a = int_type();
  let b = int_type();
  let res = check_binary(a, 57, b); // >
  if res.kind != 2 { return 1; }
  return 0;
}

pub fn test_return_compatible() -> Int {
  let val = int_type();
  let expected = int_type();
  let res = check_return(val, expected);
  if res.kind != 5 { return 1; }
  return 0;
}

pub fn test_return_mismatch() -> Int {
  let val = bool_type();
  let expected = int_type();
  let res = check_return(val, expected);
  if res.kind != 9 { return 1; }
  return 0;
}

pub fn test_let_inference() -> Int {
  let val = int_type();
  let annot = CheckedType{ kind: 0, name: 0 }; // no annotation
  let res = check_let(val, annot);
  if res.kind != 1 { return 1; }
  return 0;
}

} // end module checker

// ============================================================================
// Module exports
// ============================================================================
use checker.CheckedType;
use checker.int_type;
use checker.bool_type;
use checker.error_type;
use checker.void_type;
use checker.types_compatible;
use checker.check_binary;
use checker.check_return;
use checker.check_let;
use checker.test_int_compatible;
use checker.test_error_compatible;
use checker.test_arithmetic_int;
use checker.test_comparison_bool;
use checker.test_return_compatible;
use checker.test_return_mismatch;
use checker.test_let_inference;

// ============================================================================
// Entry point -- runs all tests
// ============================================================================
fn main() -> Int {
  var exit = 0;

  exit = test_int_compatible();
  if exit != 0 { return 100 + exit; }

  exit = test_error_compatible();
  if exit != 0 { return 200 + exit; }

  exit = test_arithmetic_int();
  if exit != 0 { return 300 + exit; }

  exit = test_comparison_bool();
  if exit != 0 { return 400 + exit; }

  exit = test_return_compatible();
  if exit != 0 { return 500 + exit; }

  exit = test_return_mismatch();
  if exit != 0 { return 600 + exit; }

  exit = test_let_inference();
  if exit != 0 { return 700 + exit; }

  return 0;
}
