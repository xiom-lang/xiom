// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m65_shift_semantics

use xiom.math;
use xiom.io;

fn main() -> Int {
  if math.shl(1, 10) != 1024 { io.println("shl10"); return 1; }
  if math.shl(5, -1) != 5 { io.println("shlneg"); return 2; }
  if math.shl(1, 100) != 0 { io.println("shl100"); return 3; }
  if math.shl(2, 63) != 0 { io.println("shl63even"); return 4; }
  if math.shl(1, 63) >= 0 { io.println("shl63odd"); return 5; }
  if math.shl(1, 63) != -9223372036854775807 - 1 { io.println("shlintmin"); return 6; }
  if math.shr(-8, 1) != -4 { io.println("shrneg"); return 7; }
  if math.shr(-1, 100) != -1 { io.println("shr100neg"); return 8; }
  if math.shr(8, 1) != 4 { io.println("shrpos"); return 9; }
  if math.shr(5, -2) != 5 { io.println("shrnegcount"); return 10; }
  if math.shr(-8, 63) != -1 { io.println("shr63neg"); return 11; }
  if math.shr(7, 63) != 0 { io.println("shr63pos"); return 12; }
  io.println("shifts ok");
  return 0;
}