// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S20: Serialization -- binary format encode/decode patterns
type BinHeader = { magic: Int; version: Int; flags: Int; size: Int; }
type SerValue = { tag: Int; int_val: Int; float_val: Float64; str_len: Int; }
fn make_header(magic: Int, ver: Int, flags: Int, size: Int) -> BinHeader {
  return BinHeader{ magic: magic; version: ver; flags: flags; size: size; };
}
fn make_value(tag: Int, iv: Int, fv: Float64, sl: Int) -> SerValue {
  return SerValue{ tag: tag; int_val: iv; float_val: fv; str_len: sl; };
}
fn validate_magic(h: BinHeader, expected: Int) -> Bool {
  return h.magic == expected;
}
fn is_version_ok(h: BinHeader, min_ver: Int, max_ver: Int) -> Bool {
  return h.version >= min_ver && h.version <= max_ver;
}
fn is_little_endian(flags: Int) -> Bool {
  return flags & 1 == 1;
}
fn is_64bit(flags: Int) -> Bool {
  return flags & 2 == 2;
}
fn serialized_size(v: SerValue) -> Int {
  if v.tag == 0 { return 4; }
  if v.tag == 1 { return 8; }
  if v.tag == 2 { return v.str_len + 4; }
  return v.str_len + 4;
}
fn main() -> Int {
  var h = make_header(0x58494F4D, 1, 3, 128);
  if !validate_magic(h, 0x58494F4D) { return 1; }
  if validate_magic(h, 0) { return 2; }
  if !is_version_ok(h, 1, 5) { return 3; }
  if is_version_ok(h, 3, 10) { return 4; }
  if !is_little_endian(3) { return 5; }
  if !is_64bit(3) { return 6; }
  if is_64bit(1) { return 7; }
  var vi = make_value(0, 42, 0.0, 0);
  var vf = make_value(1, 0, 3.14, 0);
  var vs = make_value(2, 0, 0.0, 16);
  if serialized_size(vi) != 4 { return 8; }
  if serialized_size(vf) != 8 { return 9; }
  if serialized_size(vs) != 20 { return 10; }
  return 0;
}
