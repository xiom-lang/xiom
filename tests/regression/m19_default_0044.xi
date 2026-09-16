// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0044

interface Descriptor {
  fn describe(&self) -> Str { return get_name() + ":" + get_label(); }
  fn get_name(&self) -> Str;
  fn get_label(&self) -> Str;
}

type Widget = { name: Str; label: Str; }

fn Widget.describe(self) -> Str { return self.get_name() + ":" + self.get_label(); }


fn Widget.get_name(&self) -> Str { return name; }

fn Widget.get_label(&self) -> Str { return label; }

fn main() -> Int {
  var w: Widget = Widget{ name: "btn", label: "submit" };
  if w.get_name() == "btn" && w.get_label() == "submit" && w.describe() == "btn:submit" { return 0; }
  return 1;
}
