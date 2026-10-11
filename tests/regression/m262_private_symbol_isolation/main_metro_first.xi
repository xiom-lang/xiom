// m262 (REL-20261011-1052): same lock as main_farm_first.xi with the import
// order REVERSED (metro before farm). Both orders must exit 0: each module's
// body reads its OWN private module-level consts, and the metro empty-input
// digest stays 0x13C52518F4DB1E2A in either registration order.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m262_metro_first

use m262_metro as metro;
use m262_farm as farm;

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
