// E2E regression: module-level mutable var global persists across calls.
module e2e_module_global

var counter: Int = 0;

fn bump() { counter = counter + 1; }

fn main() -> Int {
  counter = 5;
  bump();
  bump();
  if counter == 7 { return 0; }
  return 1;
}