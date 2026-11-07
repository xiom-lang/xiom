// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// REL-20261106-1840-pulse: copy-modify-write of a nested aggregate field on a
// MODULE-GLOBAL Vec[Struct] emitted invalid IR on v0.65.6:
//
//   %tmp173 = extractvalue %struct.Vec 0, 0
//   error: integer constant must have integer type
//
// `Vec.push` recorded a bogus `local_vec_elem[g_regs] = "Vec[Int]"` for a
// struct-element module global (the global's real type lives in
// `global_xiom_types`, not `local_vec_elem`). Every `g_regs[i]` read then took
// the nested-Vec path (`memcpy 32` + `load %struct.Vec`), so
// `g_regs[0].entries[idx].h` lost its aggregate type; the field write-back
// fell to `compile_lvalue`'s Index fallback with a "0" container value and
// emitted `emit_vec_store_fields("0", ...)` -> the literal operand above.
//
// This fixture locks the read, in-place reset through `&mut`, the write-back,
// and the re-read through the module global.
module rel20261106_nested_vec_struct_copy

type Histogram = { count: Int; sum: Int; }
type MetricEntry = { h: Histogram; }
type Registry = { entries: Vec[MetricEntry]; }

fn histogram_reset(h: &mut Histogram) {
  h.count = 0;
  h.sum = 0;
}

var g_regs: Vec[Registry] = Vec[Registry].new();

fn main() -> Int {
  var reg = Registry { entries: Vec[MetricEntry].new() };
  reg.entries.push(MetricEntry { h: Histogram { count: 5; sum: 15; } });
  g_regs.push(reg);

  var idx = 0;
  var h = g_regs[0].entries[idx].h;
  if h.count != 5 { return 1; }
  if h.sum != 15 { return 2; }

  histogram_reset(&mut h);
  if h.count != 0 { return 3; }

  g_regs[0].entries[idx].h = h;
  var check = g_regs[0].entries[idx].h;
  if check.count != 0 { return 4; }
  if check.sum != 0 { return 5; }

  // The module-global Vec header and the nested Vec field must survive the
  // copy-modify-write unchanged.
  if g_regs.len() != 1 { return 6; }
  if g_regs[0].entries.len() != 1 { return 7; }
  return 0;
}
