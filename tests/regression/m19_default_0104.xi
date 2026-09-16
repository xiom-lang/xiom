// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0104

interface ByteCheck {
  fn is_byte(&self) -> Bool { var v = value(); return v >= 0i8 && v <= 127i8; }
  fn value(&self) -> Int8;
}

type Data = { x: Int8; }

fn Data.is_byte(self) -> Bool { var v = self.value(); return v >= 0i8 && v <= 127i8; }


fn Data.value(&self) -> Int8 { return x; }

fn main() -> Int {
  var d: Data = Data{ x: 100i8 };
  if d.value() != 100i8 { return 1; }
  if d.is_byte() != true { return 2; }
  return 0;
}
