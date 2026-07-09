// E2E: Enum variant construction as values + equality comparison
module e2e_enum_variant_value

type Color = enum { Red, Green, Blue }

fn main() -> Int {
  var a = Color.Red;
  var b = Color.Red;
  var c = Color.Green;
  if a == b && a != c {
    return 0;
  }
  return 1;
}
