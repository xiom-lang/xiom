// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// m77 (R16): `ptr + int` used directly as a call argument must be POINTER
// arithmetic, not Str concatenation. An unannotated `var buf = malloc(n)`
// was not typed as a pointer (extern return types were never recorded in
// fn_return_xiom), so `buf + len_a` compiled as `xiom_str_concat(buf,
// int_to_string(len_a))` and passed a heap string as the memcpy dest --
// corrupting the buffer ("foo<?>\x01" instead of "foobar"). The stdlib
// fast-path had to cast through Int as a workaround; this locks the real
// lowering.
module m77_ptr_plus_int_arg

use xiom.io;

extern "C" {
  fn malloc(size: UInt) -> *UInt8;
  fn xiom_memcpy_dispatch(dest: *UInt8, src: *UInt8, n: UInt) -> *UInt8;
  fn xiom_byte_at(s: Str, pos: Int) -> UInt8;
}

// Byte-loop first, then memcpy at `buf + len_a` (the defect shape).
fn concat_offset(a: Str, b: Str) -> Str {
  let len_a = a.len();
  let len_b = b.len();
  let total = len_a + len_b;
  unsafe {
    var buf = malloc(total + 1);
    var i = 0;
    while i < len_a {
      buf[i] = xiom_byte_at(a, i);
      i = i + 1;
    }
    xiom_memcpy_dispatch(buf + len_a, b as *UInt8, len_b as UInt);
    buf[total] = 0;
    return Str.from_cstring(buf);
  }
}

// Chained concat: each link appends at `buf + total`, locking the repeated
// dest-offset arithmetic.
fn concat_chain(n: Int) -> Str {
  var acc = "";
  var k = 0;
  while k < n {
    var len = acc.len();
    unsafe {
      var buf = malloc(len + 3);
      xiom_memcpy_dispatch(buf, acc as *UInt8, len as UInt);
      xiom_memcpy_dispatch(buf + len, "ab" as *UInt8, 2 as UInt);
      buf[len + 2] = 0;
      acc = Str.from_cstring(buf);
    }
    k = k + 1;
  }
  return acc;
}

fn main() -> Int {
  if concat_offset("foo", "bar") != "foobar" { return 1; }
  if concat_offset("", "xyz") != "xyz" { return 2; }
  if concat_offset("abc", "") != "abc" { return 3; }
  if concat_chain(0) != "" { return 4; }
  if concat_chain(1) != "ab" { return 5; }
  if concat_chain(4) != "abababab" { return 6; }
  io.println("M77 OK");
  return 0;
}
