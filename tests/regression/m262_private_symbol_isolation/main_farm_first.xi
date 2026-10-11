// m262 (REL-20261011-1052): module-private module-level consts must stay per
// module. Numeric witness (metro imported AFTER farm): metrohash64(empty,0)
// must be 0x13C52518F4DB1E2A (1424585646250073642); pre-fix the bare const
// slot was last-registered-wins, metro's body substituted FARM's _K0/_K1/_K2
// and returned 0xD61B291421481536 (15427970114922222902). The const_tag pins
// catch the reverse order too (each module must see its OWN constants).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m262_farm_first

use m262_farm as farm;
use m262_metro as metro;

fn main() -> Int {
  let eb = Vec[UInt8].new();
  if metro.metrohash64(&eb, 0 as UInt64) != 0x13C52518F4DB1E2A {
    return 1;
  }
  if farm.farmhash64(&eb) != 0 {
    return 2;
  }
  if metro.metro_tag() != 0x16E3340F {
    return 3;
  }
  if farm.farm_tag() != 0xEDD6140806C3831B {
    return 4;
  }
  return 0;
}
