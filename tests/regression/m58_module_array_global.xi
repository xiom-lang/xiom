// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M58: module-level mutable array mis-materialization -- stdlib finding
// 3b-2 #8. `var _tbl: [256]Int;` at module scope with runtime INDEX writes
// then reads compiled by loading the whole [N x T] global VALUE into a
// fresh stack alloca and indexing the COPY: writes were lost and reads saw
// a stale snapshot (crc tables read back all zeros; "backing undersized"
// AV family near page boundaries). Fix: GEP the real global directly in
// both the index-read (expr.rs) and index-write (stmt.rs) arms.
module m58_module_array_global

var _t: [256]Int;
var _s: [128]Int;

fn fill() {
  var i = 0;
  while i < 256 {
    _t[i] = i * 3;
    i = i + 1;
  }
  i = 0;
  while i < 128 {
    _s[i] = i * 7;
    i = i + 1;
  }
}

fn main() -> Int {
  if _t[1] == 0 {
    fill();
  }
  var s = 0;
  var i = 0;
  while i < 256 {
    s = s + _t[i];
    i = i + 1;
  }
  // sum(0..255) * 3 = 32640 * 3 = 97920
  if s != 97920 { return 1; }
  if _t[0] != 0 { return 2; }
  if _t[255] != 765 { return 3; }
  var t = 0;
  i = 0;
  while i < 128 {
    t = t + _s[i];
    i = i + 1;
  }
  // sum(0..127) * 7 = 8128 * 7 = 56896
  if t != 56896 { return 4; }
  if _s[127] != 889 { return 5; }
  // read-modify-write through the global
  _t[10] = _t[10] + 1;
  if _t[10] != 31 { return 6; }
  return 0;
}
