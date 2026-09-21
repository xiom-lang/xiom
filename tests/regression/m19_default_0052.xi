// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0052

interface Meta {
  fn meta(&self) -> Str { return name() + " v" + version(); }
  fn name(&self) -> Str;
  fn version(&self) -> Str;
}

type App = { n: Str; ver: Str; }

fn App.meta(self) -> Str { return self.name() + " v" + self.version(); }


fn App.name(&self) -> Str { return n; }

fn App.version(&self) -> Str { return ver; }

type Lib = { n: Str; ver: Str; }

fn Lib.name(&self) -> Str { return n; }

fn Lib.version(&self) -> Str { return ver; }

fn main() -> Int {
  var a: App = App{ n: "App", ver: "1.0" };
  var l: Lib = Lib{ n: "Lib", ver: "2.5" };
  if a.meta() == "App v1.0" && l.meta() == "Lib v2.5" { return 0; }
  return 1;
}
