// XIOM — Self-Hosted Codegen
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module codegen {

pub fn llvm_type_kind(xiom_ty: Int) -> Int {
  if xiom_ty == 1 { return 0; }
  if xiom_ty == 3 { return 1; }
  if xiom_ty == 4 { return 2; }
  return 0;
}

pub fn compile_simple() -> Int {
  var count = 0;
  count = count + 1;
  count = count + 1;
  count = count + 1;
  count = count + 1;
  return count;
}

pub fn test_types() -> Int {
  if llvm_type_kind(1) != 0 { return 1; }
  if llvm_type_kind(3) != 1 { return 2; }
  return 0;
}

pub fn test_compile() -> Int {
  if compile_simple() != 4 { return 1; }
  return 0;
}

} // end module codegen

use codegen.test_types;
use codegen.test_compile;

fn main() -> Int {
  var exit = test_types();
  if exit != 0 { return 100 + exit; }
  exit = test_compile();
  if exit != 0 { return 200 + exit; }
  return 0;
}
