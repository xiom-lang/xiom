// m110 (L6-40 / playground C17 residue): a module-scoped generic FACTORY has
// no local evidence at its call site -- `var runner = plugin_runner.create()`
// fixed T only through the LATER `plugin_runner.add_plugin(&mut runner,
// EchoPlugin{})`. Single-pass emission inferred T=[] (the `0` fallback) and
// then mono'd `run_all` with T=Int -> C001 "Int does not implement Plugin".
// The bounded evidence pre-pass resolves the factory's type arg from the
// later call and seeds the binding's container type.
module m110.main

use xiom.io;

use xiom.string;
interface Plugin {
  fn name(&self) -> Str;
  fn execute(&self, input: Str) -> Str;
}

type EchoPlugin = {}
fn EchoPlugin.name(&self) -> Str { "Echo" }
fn EchoPlugin.execute(&self, input: Str) -> Str {
  string.str_concat(input.clone(), string.str_concat(" ... ", input.clone()))
}

module plugin_runner {
  type Runner[T: Plugin] = {
    plugins: Vec[T];
  }

  pub fn create[T: Plugin]() -> Runner[T] {
    Runner{ plugins: Vec[T].new() }
  }

  pub fn add_plugin[T: Plugin](r: &mut Runner[T], p: T) {
    r.plugins.push(p);
  }

  pub fn run_all[T: Plugin](r: &Runner[T], input: Str) {
    var i = 0;
    while i < r.plugins.len() {
      let p = r.plugins.get(i).unwrap();
      let output = p.execute(input.clone());
      io.println(p.name());
      io.println(output.to_str());
      i += 1;
    };
  }
}

fn main() -> Int {
  var runner = plugin_runner.create();
  plugin_runner.add_plugin(&mut runner, EchoPlugin{});
  plugin_runner.run_all(&runner, "Hello");
  return 0;
}
