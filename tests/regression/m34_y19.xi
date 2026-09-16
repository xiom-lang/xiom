// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y19: generic + contract + Result + module + impl + string operations
enum Kind { Full, Half, Quarter }
fn half_it(x: Int) -> Int { return x / 2; }
fn quarter_it(x: Int) -> Int { return x / 4; }
fn process[T](x: Int, kind: Kind) -> Result[Int, Str]
  requires: x > 0
{
  match kind {
    Full => Ok(x),
    Half => Ok(half_it(x)),
    Quarter => Ok(quarter_it(x)),
  }
}
interface Processable { fn run(self) -> Int; }
type Data = { value: Int; }
impl Processable for Data {
  fn run(self) -> Int { return self.value; }
}
module engine_mod {
  pub fn do_process(x: Int, k: Kind) -> Result[Int, Str] { return process(x, k); }
  pub fn via_data(d: Data) -> Int { return d.run(); }
}
use engine_mod.do_process;
use engine_mod.via_data;
fn main() -> Int {
  match do_process(100, Kind.Full) {
    Ok(v) => { if v != 100 { return 1; } }
    Err(_) => { return 2; }
  }
  match do_process(100, Kind.Half) {
    Ok(v) => { if v != 50 { return 3; } }
    Err(_) => { return 4; }
  }
  match do_process(40, Kind.Quarter) {
    Ok(v) => { if v != 10 { return 5; } }
    Err(_) => { return 6; }
  }
  var d = Data{ value: 42; };
  if via_data(d) != 42 { return 7; }
  return 0;
}
