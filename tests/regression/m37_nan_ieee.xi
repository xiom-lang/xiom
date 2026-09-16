// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_nan_ieee
// BUG 19 regression: IEEE-754 NaN/Inf semantics + Str + Float64 concat.
// 1) float `!=` must lower to fcmp UNE -- `x != x` is TRUE for NaN (the old
//    `fcmp one` made it false, so is_nan() was impossible).
// 2) NaN-producing ops (0.0/0.0, inf*0.0, inf-inf) must yield real NaN values
//    (the old concat path inttoptr'd the FP bits -> AV / garbage sentinels).
// 3) Str + Float64 concat must FORMAT via xiom_double_to_string ("nan",
//    "inf", "-inf", shortest round-trip for finite values).

use xiom.io;

fn main() -> Int {
  // NaN production: 0.0/0.0
  var a = 0.0;
  var b = 0.0;
  var nan1 = a / b;
  if !(nan1 != nan1) { io.println("fail: 0/0 not NaN"); return 1; }
  if nan1 == nan1 { io.println("fail: NaN == NaN true"); return 2; }

  // +inf / -inf production
  var pos = 1.0 / 0.0;
  var neg = -1.0 / 0.0;
  if !(pos > 0.0) { return 3; }
  if !(neg < 0.0) { return 4; }
  if pos != pos { return 5; }
  if neg != neg { return 6; }

  // inf * 0.0 and inf - inf are NaN
  var nan2 = pos * 0.0;
  if !(nan2 != nan2) { return 7; }
  var nan3 = pos - pos;
  if !(nan3 != nan3) { return 8; }

  // Ordering comparisons with NaN are all false (olt/ole/ogt/oge)
  if nan1 < 0.0 { return 9; }
  if nan1 > 0.0 { return 10; }
  if nan1 <= 0.0 { return 11; }
  if nan1 >= 0.0 { return 12; }

  // Normal comparisons still correct
  var x = 1.0;
  var y = 2.0;
  if !(x != y) { return 13; }
  if x != x { return 14; }
  if !(y == y) { return 15; }
  if x == y { return 16; }

  // Float32 NaN path
  var f1: Float32 = 0.0;
  var f2: Float32 = 0.0;
  var fnan = f1 / f2;
  if !(fnan != fnan) { return 17; }

  // Str + Float64 concat: formats, never inttoptrs
  if ("nan? " + nan1) != "nan? nan" { io.println("fail: nan concat"); return 18; }
  if ("inf: " + pos) != "inf: inf" { return 19; }
  if ("neg: " + neg) != "neg: -inf" { return 20; }
  if ("pi? " + 3.14) != "pi? 3.14" { return 21; }
  if ("z: " + 0.0) != "z: 0" { return 22; }
  if ("h: " + 0.5) != "h: 0.5" { return 23; }

  io.println("nan ok");
  return 0;
}
