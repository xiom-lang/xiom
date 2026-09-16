// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y15: if-else chain + compound assign + Result + enum + match + module + impl
type Env = { temperature: Int; humidity: Int; }
enum Climate { Hot, Cold, Mild }
fn classify[T](e: Env) -> Climate {
  if e.temperature > 30 { return Climate.Hot; }
  if e.temperature < 10 { return Climate.Cold; }
  return Climate.Mild;
}
fn adjust[T](e: Env, target: Climate) -> Result[Env, Str]
  requires: e.temperature >= 0
  requires: e.humidity >= 0
{
  var t = e.temperature;
  var h = e.humidity;
  match target {
    Hot => { t = t + 10; h = h - 5; }
    Cold => { t = t - 8; h = h + 10; }
    Mild => { t = t + 2; h = h + 3; }
  }
  return Ok(Env{ temperature: t; humidity: h; });
}
module env_mod {
  pub fn do_classify(e: Env) -> Climate { return classify(e); }
  pub fn do_adjust(e: Env, t: Climate) -> Result[Env, Str] { return adjust(e, t); }
}
use env_mod.do_classify;
use env_mod.do_adjust;
interface Climatizer { fn climate(self) -> Climate; }
impl Climatizer for Env {
  fn climate(self) -> Climate { return classify(self); }
}
fn main() -> Int {
  var e = Env{ temperature: 25; humidity: 50; };
  var c = do_classify(e);
  match c { Mild => {} _ => { return 1; } }
  match do_adjust(e, Climate.Hot) {
    Ok(env) => {
      if env.temperature != 35 { return 2; }
      if env.humidity != 45 { return 3; }
    }
    Err(_) => { return 4; }
  }
  if e.climate() != Climate.Mild { return 5; }
  return 0;
}
