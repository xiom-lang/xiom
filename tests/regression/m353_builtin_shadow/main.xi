// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m353 lock (REL-20261011-1053-stdlib): a DECLARED concrete `Int.hash` /
// `Bool.hash` used to be unreachable from the zero-arg method form: codegen
// lowered every scalar `.hash()` to the builtin identity before user dispatch,
// so `5.hash()` returned 5 and stdlib `false.hash()` returned 0 instead of the
// declared body. This fixture COMPILED pre-fix and silently returned the wrong
// values (fail bits 1/2 for the declarations, 64/128 for the generic relay);
// post-fix every form dispatches to the declaration. A scalar type with NO
// declared hash keeps the builtin identity (the Float64 bitcast control), and
// struct receivers keep their own method. The explicit-receiver form
// (`Int.hash(5)`) is locked separately in main_static.xi (it did not compile
// pre-fix -- T001 expected Self).
module m353_builtin_shadow.main;

use xiom.io;

pub interface Hasherish {
  fn hash(self) -> Int;
}

pub fn Int.hash(self) -> Int {
  return 424242;
}

pub fn Bool.hash(self) -> Int {
  return 177573;
}

pub type Widget = { n: Int; }

pub fn Widget.hash(self) -> Int {
  return self.n * 10 + 7;
}

fn generic_hash[T: Hasherish](value: T) -> Int {
  return value.hash();
}

pub fn main() -> Int {
  var fail = 0;
  let i = 5;
  let b = false;
  let w = Widget { n: 3 };
  io.println("int=" + i.hash().to_str());
  io.println("bool=" + b.hash().to_str());
  io.println("widget=" + w.hash().to_str());
  if i.hash() != 424242 { fail = fail + 1; }
  if b.hash() != 177573 { fail = fail + 2; }
  if w.hash() != 37 { fail = fail + 4; }
  // Builtin fallback stays for a scalar with no declared hash: IEEE-754
  // 1.5 bits = 0x3FF8000000000000.
  let f = 1.5;
  io.println("float=" + f.hash().to_str());
  if f.hash() != 4609434218613702656 { fail = fail + 32; }
  if generic_hash(9) != 424242 { fail = fail + 64; }
  if generic_hash(false) != 177573 { fail = fail + 128; }
  io.println("m353 hash fail_bits=" + fail.to_str());
  return fail;
}
