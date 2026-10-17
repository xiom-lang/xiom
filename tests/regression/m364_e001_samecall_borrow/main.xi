// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m364 (REL-20261017-1223-orbitdb): E001 must not report a by-value
// argument followed by a borrow of the SAME local in a later argument of the
// same call. rustc lowers every call operand (the nested borrow included)
// before the Call consumes the by-value operands
// (rustc_mir_build/src/builder/expr/into.rs:485-515;
// builder/scope.rs:1557 `record_operands_moved`), so the borrow is not a
// use-after-move. Pre-fix the ONE borrow site warned TWICE: check_use at the
// inner identifier span and read_borrow at the `&` span. Codegen passes Vecs
// as copies, so the values stay valid at runtime. No E001 in either default
// or --strict mode.
module m364_e001_samecall_borrow;

fn take(d: Vec[Int], r: &Vec[Int]) -> Int { return r.len(); }
fn ck(v: &Vec[Int]) -> Int { return v.len(); }
fn take_n(d: Vec[Int], n: Int) -> Int { return d.len() + n; }

// Minimal 2-line shape: by-value argument, then `&same` in the same call.
fn minimal() -> Int {
  let bytes: Vec[Int] = Vec[Int].new();
  return take(bytes, &bytes);
}

// Nested-checksum shape: the later argument is a CALL taking `&bytes`.
fn nested() -> Int {
  let bytes: Vec[Int] = Vec[Int].new();
  return take_n(bytes, ck(&bytes));
}

fn main() -> Int {
  if minimal() != 0 { return 1; }
  if nested() != 0 { return 2; }
  return 0;
}
