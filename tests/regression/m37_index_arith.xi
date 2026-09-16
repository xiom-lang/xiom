// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_index_arith
// Parser regression (docs/COMPILER_BUGS.md): `bits[L - 1]` -- an index whose
// expression starts with an UPPERCASE ident -- was mis-parsed as explicit
// generic call args (`fn[Type](...)`) and errored ("expected ']', found -").
// The generic-args heuristic now commits only when `]` is followed by `(`.

fn f(bits: &Vec[Int]) -> Int {
  var L = bits.len();
  if L == 0 { return 0; }
  if bits[L - 1] == 0 { return 1; }
  return 2;
}

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(0);
  v.push(5);
  var r = f(&v);
  if r != 2 { return 1; }
  // Lowercase-ident arithmetic index still works.
  var i = 1;
  if v[i + 1 - 1] != 5 { return 2; }
  return 0;
}
