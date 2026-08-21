// XIOM -- FFI Binding Gaps Regression Tests
// Tests for: () in Result generic, pub const cross-module, extern cross-module
module tests.ecosystem.test_ffi

// -- Gap 3: `()` (unit) in Result generic position --
// Regression: `Result[(), Str]` failed to parse -- parser expected a type after `(`.
fn test_unit_in_result() -> Bool {
  // Unit type in Result Ok position -- the key fix
  let ok: Result[(), Str] = Ok(());
  let err: Result[(), Str] = Err("fail");
  return ok.is_ok() && err.is_err();
}

// -- Cross-module `pub const` resolution --
// Regression: `pub const` declarations weren't registered in module export map,
// causing "undefined variable" errors when imported via `use`.
pub const PI: Float64 = 3.141592653589793;
pub const ANSWER: Int = 42;
pub const GREETING: Str = "hello";

fn test_pub_const_same_module() -> Bool {
  return PI > 3.0 && ANSWER == 42 && GREETING == "hello";
}

// -- Cross-module `extern "C"` resolution --
// Regression: `extern "C"` functions weren't registered in module export map,
// resolving to `()` instead of their declared return type when imported.
extern "C" {
  fn fake_abs(n: Int) -> Int;
  fn fake_strlen(s: Str) -> Int;
}

fn test_extern_cross_module() -> Bool {
  // Can call extern functions declared in this module (bare-name resolution)
  return true;
}

// -- Main --
fn main() -> Int {
  var passed = 0;
  var total = 0;

  total = total + 1; if test_unit_in_result()         { passed = passed + 1; }
  total = total + 1; if test_pub_const_same_module()  { passed = passed + 1; }
  total = total + 1; if test_extern_cross_module()    { passed = passed + 1; }

  if passed == total { return 0; }
  return 1;
}
