// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S29: Debug info generation -- DWARF-like line table entries
type DebugLine = { addr: Int; file_id: Int; line: Int; col: Int; }
type DebugFile = { id: Int; name: Str; dir_index: Int; }
type DebugCU = { id: Int; name: Str; low_pc: Int; high_pc: Int; }
fn make_debug_line(addr: Int, fid: Int, line: Int, col: Int) -> DebugLine {
  return DebugLine{ addr: addr; file_id: fid; line: line; col: col; };
}
fn make_debug_file(id: Int, name: Str, dir: Int) -> DebugFile {
  return DebugFile{ id: id; name: name; dir_index: dir; };
}
fn make_debug_cu(id: Int, name: Str, low: Int, high: Int) -> DebugCU {
  return DebugCU{ id: id; name: name; low_pc: low; high_pc: high; };
}
fn line_covers_addr(line: DebugLine, addr: Int) -> Bool {
  return addr >= line.addr;
}
fn file_matches_id(f: DebugFile, fid: Int) -> Bool {
  return f.id == fid;
}
fn cu_range(cu: DebugCU) -> Int {
  return cu.high_pc - cu.low_pc;
}
fn is_debug_line_end(line: DebugLine) -> Bool {
  return line.line == 0 && line.col == 0 && line.addr == 0;
}
fn main() -> Int {
  var l1 = make_debug_line(0x1000, 1, 10, 5);
  var l2 = make_debug_line(0x1010, 1, 11, 3);
  var l3 = make_debug_line(0x1020, 1, 12, 1);
  var end = make_debug_line(0, 0, 0, 0);
  if l1.line != 10 || l1.col != 5 { return 1; }
  if l2.addr != 0x1010 { return 2; }
  var f1 = make_debug_file(1, "test.xi", 0);
  if !file_matches_id(f1, 1) { return 3; }
  if file_matches_id(f1, 2) { return 4; }
  var cu = make_debug_cu(1, "test_module", 0x1000, 0x2000);
  if cu_range(cu) != 0x1000 { return 5; }
  if !line_covers_addr(l1, 0x1005) { return 6; }
  if !is_debug_line_end(end) { return 7; }
  if is_debug_line_end(l1) { return 8; }
  return 0;
}
