// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E03: 50-field struct -- large struct definition, initialization, and field access
type BigStruct = { f1: Int; f2: Int; f3: Int; f4: Int; f5: Int; f6: Int; f7: Int; f8: Int; f9: Int; f10: Int; f11: Int; f12: Int; f13: Int; f14: Int; f15: Int; f16: Int; f17: Int; f18: Int; f19: Int; f20: Int; f21: Int; f22: Int; f23: Int; f24: Int; f25: Int; f26: Int; f27: Int; f28: Int; f29: Int; f30: Int; f31: Int; f32: Int; f33: Int; f34: Int; f35: Int; f36: Int; f37: Int; f38: Int; f39: Int; f40: Int; f41: Int; f42: Int; f43: Int; f44: Int; f45: Int; f46: Int; f47: Int; f48: Int; f49: Int; f50: Int; }
fn main() -> Int {
  var s = BigStruct{ f1: 1; f2: 2; f3: 3; f4: 4; f5: 5; f6: 6; f7: 7; f8: 8; f9: 9; f10: 10; f11: 11; f12: 12; f13: 13; f14: 14; f15: 15; f16: 16; f17: 17; f18: 18; f19: 19; f20: 20; f21: 21; f22: 22; f23: 23; f24: 24; f25: 25; f26: 26; f27: 27; f28: 28; f29: 29; f30: 30; f31: 31; f32: 32; f33: 33; f34: 34; f35: 35; f36: 36; f37: 37; f38: 38; f39: 39; f40: 40; f41: 41; f42: 42; f43: 43; f44: 44; f45: 45; f46: 46; f47: 47; f48: 48; f49: 49; f50: 50 };
  var total = s.f1 + s.f2 + s.f3 + s.f4 + s.f5 + s.f6 + s.f7 + s.f8 + s.f9 + s.f10 + s.f11 + s.f12 + s.f13 + s.f14 + s.f15 + s.f16 + s.f17 + s.f18 + s.f19 + s.f20 + s.f21 + s.f22 + s.f23 + s.f24 + s.f25 + s.f26 + s.f27 + s.f28 + s.f29 + s.f30 + s.f31 + s.f32 + s.f33 + s.f34 + s.f35 + s.f36 + s.f37 + s.f38 + s.f39 + s.f40 + s.f41 + s.f42 + s.f43 + s.f44 + s.f45 + s.f46 + s.f47 + s.f48 + s.f49 + s.f50;
  if total != 1275 { return 1; }
  if s.f1 != 1 { return 2; }
  if s.f25 != 25 { return 3; }
  if s.f50 != 50 { return 4; }
  return 0;
}