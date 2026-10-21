// m1703 (REL-20261021-1703-pulse): function values stored in a module var
// or a Vec must keep their declared signature and aggregate return type.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// The PULSE StreamHandler shape: fn(Int, &Vec[UInt8], &Req) -> (Int, Int)
// kept in `var g: Handler` and in `Vec[Handler]`. Pre-fix:
//   - the Vec element call emitted `call i64 %fn(...)` (return type lost),
//     so both tuple fields read the same low word (b0=4 b1=4);
//   - the push did not wrap a bare fn ref for a Vec of a fn ALIAS, so the
//     element-call env convention crashed once the return type was fixed.
// The fix expands fn aliases at Vec-element resolution and wraps bare fn
// refs on push, matching array literals.

module m1703_fn_value_vec_tuple

use xiom.io;
use xiom.convert;

type Handler = fn(Int, &Vec[UInt8], &Req) -> (Int, Int);

pub type Req = {
  id: Int;
}

fn on_msg(x: Int, buf: &Vec[UInt8], req: &Req) -> (Int, Int) {
  return (x + buf.len(), req.id);
}

var g: Handler = on_msg;

pub fn main() -> Int {
  var buf = Vec[UInt8].new();
  buf.push(1);
  buf.push(2);
  let req = Req{ id: 7 };

  let d = on_msg(2, &buf, &req);
  let (d0, d1) = d;
  if d0 != 4 { return 1; }
  if d1 != 7 { return 2; }

  let e = g(2, &buf, &req);
  let (e0, e1) = e;
  if e0 != 4 { return 3; }
  if e1 != 7 { return 4; }

  var v: Vec[Handler] = Vec[Handler].new();
  v.push(on_msg);
  let h = v[0];
  let b = h(2, &buf, &req);
  let (b0, b1) = b;
  io.println("b0=" + convert.int_to_string(b0) + " b1=" + convert.int_to_string(b1));
  if b0 != 4 { return 5; }
  if b1 != 7 { return 6; }

  io.println("m1703 ok");
  return 0;
}
