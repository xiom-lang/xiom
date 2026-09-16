// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// REPRO R3 (BUG 27 #14): Int returned from a catalog unsafe block.
// io/fs.xi fs_move used this pattern: the file moved correctly but rc always
// read non-zero -- the return value through the unsafe trampoline corrupts.
module repro_unsafe_int

extern "C" {
  fn rename(old: *UInt8, new: *UInt8) -> Int32;
}

fn cstr(s: Str) -> *UInt8

pub fn rename_rc(src: Str, dst: Str) -> Int {
  unsafe {
    var r = rename(cstr(src), cstr(dst)) as Int;
    return r;
  }
}