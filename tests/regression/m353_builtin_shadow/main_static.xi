// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m353 lock (REL-20261011-1053-stdlib), explicit-receiver half: the UFCS
// static form `Int.hash(5)` / `Bool.hash(false)` failed T001 pre-fix
// ("argument 1 type mismatch: expected Self, found Int") because the declared
// method's explicit-receiver param spells `Self` while the call-site argument
// is a primitive. The stdlib relay names this form explicitly; post-fix both
// calls dispatch to the declarations.
module m353_builtin_shadow.main_static;

use xiom.io;

pub fn Int.hash(self) -> Int {
  return 424242;
}

pub fn Bool.hash(self) -> Int {
  return 177573;
}

pub fn main() -> Int {
  var fail = 0;
  io.println("int=" + Int.hash(5).to_str());
  io.println("bool=" + Bool.hash(false).to_str());
  if Int.hash(5) != 424242 { fail = fail + 1; }
  if Bool.hash(false) != 177573 { fail = fail + 2; }
  io.println("m353 static hash fail_bits=" + fail.to_str());
  return fail;
}
