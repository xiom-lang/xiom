// m320 lock (REL-20261013-1819-engine): an Int-typed pointer value at a
// named-call boundary. Pre-fix `takes8(v.as_mut_ptr() as Int)` (the engine's
// legacy `pc.as_mut_ptr() as Int` idiom) hit the B-31 value-arg
// materialization: the i64 was truncated into a 1-byte stack slot and the
// slot ADDRESS was passed, so the callee read the low byte of the stack
// address (witness: 208) instead of the buffer bytes. Pointer-bit args must
// inttoptr. Shapes: A plain typed / B inline `as Int` (the reported idiom) /
// C Int binding cast back / D-E indirect fn-pointer calls. Green = 0.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m320_int_ptr_arg;

use xiom.io;

fn mk(n: Int) -> Vec[UInt8] {
  var v: Vec[UInt8] = Vec[UInt8].new();
  var i = 0;
  while i < n {
    v.push(((i * 7 + 3) % 251 + 1) as UInt8);
    i = i + 1;
  }
  return v;
}

fn takes8(p: *UInt8) -> Int
  requires: true
{
  unsafe {
    let b0 = p[0] as Int;
    let b1 = p[1] as Int;
    let b2 = p[2] as Int;
    let b3 = p[3] as Int;
    return b0 + (b1 << 8) + ((b2 << 16) + (b3 << 24));
  }
}

fn main() -> Int {
  var v = mk(8);
  let expect = (v[0] as Int) + ((v[1] as Int) << 8) + ((v[2] as Int) << 16) + ((v[3] as Int) << 24);
  unsafe {
    // A: plain typed delivery.
    if takes8(v.as_mut_ptr()) != expect { return 1; }
    // B: inline `as Int` argument (the reported failing idiom).
    if takes8(v.as_mut_ptr() as Int) != expect { return 2; }
    // C: Int-typed variable cast back to *UInt8 at the boundary.
    let ip = v.as_mut_ptr() as Int;
    if takes8(ip as *UInt8) != expect { return 3; }
    // D: indirect fn-pointer call with the same Int dance.
    let f: fn(*UInt8) -> Int = takes8;
    if f(ip as *UInt8) != expect { return 4; }
    if f(v.as_mut_ptr() as Int) != expect { return 5; }
  }
  io.println("m320 int-ptr-arg ok");
  return 0;
}
