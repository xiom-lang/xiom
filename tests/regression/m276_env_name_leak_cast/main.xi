// m276 (REL-20261012-1418-pulse): a fn-typed PARAM of one function leaks its
// name into the program-wide `closure_locals` set; an unrelated same-named
// plain UInt32 param doing `v as Int` then compiled to
// `inttoptr(zext(v))` + load (the env field-0 convention) and AV'd -- the
// empty-store segment create of kv-mode PULSE goes through exactly this
// shape in stdlib serialize.endian write_u32_le / write_u16_le (their
// `var x = v as Int`). The As-cast / call-site env unwrap must veto a name
// whose current binding is concretely non-fn typed. Pre-fix this program
// crashes (0xC0000005, reads address 0x12345678); post-fix it exits 0.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m276_env_name_leak_cast;

use xiom.io;
use xiom.convert;

// The leak source: a fn-typed param named `v` registers `v` in the
// program-wide closure_locals set. Called from main so it is live.
fn consume(v: fn(Int) -> Int) -> Int {
  return v(7);
}

fn double(x: Int) -> Int {
  return x * 2;
}

// The victim: a plain UInt32 param sharing the leaked name. Its As-cast
// must stay an integer widening (pre-fix: inttoptr(zext(v)) -> AV).
fn write_u32_like(out: &mut Vec[UInt8], v: UInt32) {
  var x = v as Int;
  out.push((x & 0xFF) as UInt8);
  out.push(((x >> 8) & 0xFF) as UInt8);
  out.push(((x >> 16) & 0xFF) as UInt8);
  out.push(((x >> 24) & 0xFF) as UInt8);
}

fn main() -> Int {
  var fails: Int = 0;

  // 305419896 == 0x12345678 -> little-endian bytes 120, 86, 52, 18.
  var buf: Vec[UInt8] = Vec[UInt8].new();
  write_u32_like(&mut buf, 305419896);
  if buf.len() != 4 {
    fails = fails + 1;
    io.println("  [CORRUPT] A length=" + convert.int_to_string(buf.len()));
  } else {
    if (buf[0] as Int) != 120 { fails = fails + 1; io.println("  [CORRUPT] A b0=" + convert.int_to_string(buf[0] as Int)); }
    if (buf[1] as Int) != 86 { fails = fails + 1; io.println("  [CORRUPT] A b1=" + convert.int_to_string(buf[1] as Int)); }
    if (buf[2] as Int) != 52 { fails = fails + 1; io.println("  [CORRUPT] A b2=" + convert.int_to_string(buf[2] as Int)); }
    if (buf[3] as Int) != 18 { fails = fails + 1; io.println("  [CORRUPT] A b3=" + convert.int_to_string(buf[3] as Int)); }
  }

  // The fn-param path must keep working (B-007 env wrap; control).
  let got = consume(double);
  if got != 14 {
    fails = fails + 1;
    io.println("  [CORRUPT] B consume(double)=" + convert.int_to_string(got));
  }

  io.println("m276_env_name_leak_cast: " + convert.int_to_string(fails) + " corrupted shapes");
  return fails;
}
