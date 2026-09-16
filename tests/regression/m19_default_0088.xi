// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0088

interface Area {
  fn area(&self) -> Int { return width() * height(); }
  fn width(&self) -> Int;
  fn height(&self) -> Int;
}

type Rect = { w: Int; h: Int; }

fn Rect.area(self) -> Int { return self.width() * self.height(); }


fn Rect.width(&self) -> Int { return w; }

fn Rect.height(&self) -> Int { return h; }

fn main() -> Int {
  var r: Rect = Rect{ w: 7, h: 11 };
  if r.width() == 7 && r.height() == 11 && r.area() == 77 { return 0; }
  return 1;
}
