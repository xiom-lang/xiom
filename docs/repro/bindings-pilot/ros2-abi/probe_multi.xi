// ABI probe #6: multi-arg shapes (9-arg wait_set_init-like, 7-arg timer_init2-like,
// 3-arg clock_init-like) with the by-value 40B allocator passed via pointer-pretend.
// Expected: m1 8186 / m2 2166 / m3 969 (the pointer-pretend workaround channel:
// ABI-correct where aggregates are passed by reference to a caller copy).
// Parameterized by XIOM_M351_PROBE_DLL (see probe_ladder.xi).
//
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module probe_multi

use xiom.ffi.dl;
use xiom.convert;
use xiom.io;
use xiom.os.env;

fn say(s: Str) {
  io.println(s);
  io.flush_stdout();
}

fn wr_u64_le(buf: &mut Vec[UInt8], off: Int, v: Int) {
  buf[off] = (v & 255) as UInt8;
  buf[off + 1] = ((v >> 8) & 255) as UInt8;
  buf[off + 2] = ((v >> 16) & 255) as UInt8;
  buf[off + 3] = ((v >> 24) & 255) as UInt8;
  buf[off + 4] = ((v >> 32) & 255) as UInt8;
  buf[off + 5] = ((v >> 40) & 255) as UInt8;
  buf[off + 6] = ((v >> 48) & 255) as UInt8;
  buf[off + 7] = ((v >> 56) & 255) as UInt8;
}

pub fn main() -> Int {
  let dll = env.get_var("XIOM_M351_PROBE_DLL");
  if !dll.is_ok {
    say("XIOM_M351_PROBE_DLL is not set");
    return 2;
  }
  let h = dl.dl_open(dll.value);
  if !h.is_ok {
    say("load failed: " + h.error);
    return 1;
  }
  let handle = h.value;
  var al = Vec[UInt8].with_len(40);
  wr_u64_le(&mut al, 0, 11);
  wr_u64_le(&mut al, 8, 22);
  wr_u64_le(&mut al, 16, 33);
  wr_u64_le(&mut al, 24, 44);
  wr_u64_le(&mut al, 32, 55);
  let alptr = al.as_mut_ptr() as Int;

  let s9 = dl.dl_sym(handle, "probe_ws9");
  if s9.is_ok {
    say("m1 9-arg ws9 calling");
    unsafe {
      let f = s9.value as fn(*UInt8, Int, Int, Int, Int, Int, Int, *UInt8, *UInt8) -> Int;
      let r = f(1000 as *UInt8, 1, 2, 3, 4, 5, 6, 7000 as *UInt8, alptr as *UInt8);
      say("m1 returned " + convert.int_to_string(r) + " (want 8186)");
    }
  }

  let t7 = dl.dl_sym(handle, "probe_t7");
  if t7.is_ok {
    say("m2 7-arg t7 calling");
    unsafe {
      let f = t7.value as fn(*UInt8, *UInt8, *UInt8, Int, Int, *UInt8, Int) -> Int;
      let r = f(100 as *UInt8, 200 as *UInt8, 300 as *UInt8, 400, 500, alptr as *UInt8, 600);
      say("m2 returned " + convert.int_to_string(r) + " (want 2166)");
    }
  }

  let c3 = dl.dl_sym(handle, "probe_c3");
  if c3.is_ok {
    say("m3 3-arg c3 calling");
    unsafe {
      let f = c3.value as fn(Int, *UInt8, *UInt8) -> Int;
      let r = f(3, 900 as *UInt8, alptr as *UInt8);
      say("m3 returned " + convert.int_to_string(r) + " (want 969)");
    }
  }

  say("done");
  return 0;
}
