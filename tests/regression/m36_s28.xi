// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S28: JIT compilation -- code buffer allocation and relocation patterns
type JitCodeBuf = { base_addr: Int; size: Int; used: Int; executable: Bool; }
type JitReloc = { offset: Int; target: Int; kind: Int; }
fn make_code_buf(addr: Int, sz: Int) -> JitCodeBuf {
  return JitCodeBuf{ base_addr: addr; size: sz; used: 0; executable: false; };
}
fn make_reloc(off: Int, target: Int, kind: Int) -> JitReloc {
  return JitReloc{ offset: off; target: target; kind: kind; };
}
fn can_emit(buf: JitCodeBuf, bytes_needed: Int) -> Bool {
  return buf.used + bytes_needed <= buf.size;
}
fn emit_bytes(buf: JitCodeBuf, bytes: Int) -> JitCodeBuf {
  return JitCodeBuf{ base_addr: buf.base_addr; size: buf.size; used: buf.used + bytes; executable: buf.executable; };
}
fn finalize(buf: JitCodeBuf) -> JitCodeBuf {
  return JitCodeBuf{ base_addr: buf.base_addr; size: buf.size; used: buf.used; executable: true; };
}
fn is_absolute_reloc(kind: Int) -> Bool { return kind == 0; }
fn is_pc_relative(kind: Int) -> Bool { return kind == 1; }
fn is_plt_reloc(kind: Int) -> Bool { return kind == 2; }
fn reloc_size(buf: JitCodeBuf, rel: JitReloc) -> Int {
  if rel.kind == 0 { return 8; }
  if rel.kind == 1 { return 4; }
  return 4;
}
fn main() -> Int {
  var buf = make_code_buf(0x1000, 4096);
  if buf.base_addr != 0x1000 || buf.size != 4096 { return 1; }
  if buf.used != 0 { return 2; }
  if buf.executable { return 3; }
  var buf2 = emit_bytes(buf, 128);
  if buf2.used != 128 { return 4; }
  if !can_emit(buf2, 3000) { return 5; }
  var buf3 = finalize(buf2);
  if !buf3.executable { return 6; }
  var r1 = make_reloc(16, 0x2000, 0);
  var r2 = make_reloc(32, 0x3000, 1);
  var r3 = make_reloc(48, 0x4000, 2);
  if !is_absolute_reloc(r1.kind) { return 7; }
  if !is_pc_relative(r2.kind) { return 8; }
  if !is_plt_reloc(r3.kind) { return 9; }
  if reloc_size(buf3, r1) != 8 { return 10; }
  if reloc_size(buf3, r2) != 4 { return 11; }
  if !can_emit(buf3, 0) { return 12; }
  return 0;
}
