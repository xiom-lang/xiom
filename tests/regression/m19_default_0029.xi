// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0029

interface ThresholdCheck {
  fn threshold(&self) -> Int8 { return 100i8; }
  fn value(&self) -> Int8;
}

type Sensor = { reading: Int8; }

fn Sensor.threshold(self) -> Int8 { return 100i8; }


fn Sensor.value(&self) -> Int8 { return reading; }

fn main() -> Int {
  var s: Sensor = Sensor{ reading: 50i8 };
  if s.value() == 50i8 && s.threshold() == 100i8 { return 0; }
  return 1;
}
