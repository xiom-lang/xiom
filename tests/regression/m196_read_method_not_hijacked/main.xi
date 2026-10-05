// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m196 (Pulse C-PULSE-01 / R-8): a user method named `read` with exactly one
// argument must call the real method, not the raw-pointer codegen builtin.
// Pre-fix: the builtin fired on the `&mut Vec` argument (Vec*), the method
// body never ran (no push) and the result was garbage.
type Sock = { n: Int; }

fn Sock.read(self, buf: &mut Vec[UInt8]) -> Int {
  buf.push(65u8);
  return 7;
}

fn main() -> Int {
  let s = Sock{ n: 1 };
  var v: Vec[UInt8] = Vec[UInt8].new();
  let r = s.read(&mut v);
  if r != 7 { return 1; }
  if v.len() != 1 { return 2; }
  if v[0] != 65u8 { return 3; }
  return 0;
}
