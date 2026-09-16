// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0105

interface Int8Fit {
  fn fits_in_int8(&self) -> Bool { var v = value(); return v >= -128i16 && v <= 127i16; }
  fn value(&self) -> Int16;
}

type Data = { x: Int16; }

fn Data.fits_in_int8(self) -> Bool { var v = self.value(); return v >= -128i16 && v <= 127i16; }


fn Data.value(&self) -> Int16 { return x; }

fn main() -> Int {
  var d: Data = Data{ x: 100i16 };
  if d.value() != 100i16 { return 1; }
  if d.fits_in_int8() != true { return 2; }
  return 0;
}
