// m0717 (REL-20261021-0717-pulse): module-scope Vec initializer emitted a
// data-pointer load through the STRUCT pointer (`load i8*, i8** %struct`),
// which typed-pointer LLVM (clang 14 arm64, Apple clang arm64) rejects.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// The loader now goes through a field-0 GEP. The e2e test emits arm IR and
// asserts the ginit data load reads an `i8**` produced by a `%struct.Vec`
// GEP, so the check does not depend on a typed-pointer clang being installed.

module m0717_module_vec_init

var g: Vec[Str] = Vec[Str].new();

fn main() -> Int {
  g.push("x");
  g.push("y");
  if g.len() != 2 { return 1; }
  return 0;
}
