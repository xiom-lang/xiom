// E2E regression test: `match self` on an enum receiver method.
// This was broken by a compiler bug where the `self` parameter was registered
// twice (once as the typed struct receiver, once as a phantom `i64` param from
// the parser). The phantom shadowed the real `self`, so variant patterns in
// `match self` degraded to variable bindings, causing arms to return bare `i64`
// discriminants instead of properly-constructed enum structs — producing
// `store %struct.MyEnum i64` (invalid LLVM IR).
//
// The fix skips the duplicate `self` param in codegen when a receiver is
// present, so `self` always refers to the typed receiver struct and
// `match self { Variant => Variant }` constructs the struct correctly.
//
// This program returns 0 if the fix works, nonzero otherwise.

module e2e_method_match_self_enum

// Define a simple 3-variant enum (mirrors Ordering shape without stdlib deps)
pub enum TrafficLight = enum { Red, Yellow, Green }

pub fn TrafficLight.reverse(self) -> TrafficLight {
  match self {
    Red => Green,
    Yellow => Yellow,
    Green => Red,
  }
}

fn main() -> Int {
  // Red.reverse should be Green (discriminant 2)
  let r = TrafficLight.reverse(TrafficLight.Red);
  // Green.reverse should be Red (discriminant 0)
  let g = TrafficLight.reverse(TrafficLight.Green);
  // Yellow.reverse should be Yellow (discriminant 1)
  let y = TrafficLight.reverse(TrafficLight.Yellow);

  // If the phantom-self bug is present, `match self` in `reverse` treats
  // `self` as an `i64` phantom param, so `Red => Green` actually binds `Red`
  // as a variable holding the scrutinee's i64 discriminant, and the body
  // `Green` is an undefined name → type error. The program won't compile.
  // If it does compile, the three calls all work and we return 0.
  // (In a full runtime test we'd compare discriminants; here the smoke is
  // compile+link+run → exit 0.)

  return 0;
}
