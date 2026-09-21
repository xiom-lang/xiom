// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E16: Unicode in strings — emoji, CJK, Arabic
fn main() -> Int {
  var emoji = "😀🎉🚀🌟";
  var cjk = "你好世界";
  var arabic = "مرحبا بالعالم";
  var mixed = "Hello 世界 🌍";
  var ok = 0;
  if emoji != "😀🎉🚀🌟" { ok = 1; }
  if cjk != "你好世界" { ok = 2; }
  if arabic != "مرحبا بالعالم" { ok = 3; }
  if mixed != "Hello 世界 🌍" { ok = 4; }
  return ok;
}
