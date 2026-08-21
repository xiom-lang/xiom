// M36-X03: External type references -- type aliases and cross-referencing types
type Celsius = Float64;
type Fahrenheit = Float64;
type TempReading = { c: Float64; f: Float64; }
fn to_fahrenheit(c: Float64) -> Float64 {
  return c * 1.8 + 32.0;
}
fn to_celsius(f: Float64) -> Float64 {
  return (f - 32.0) / 1.8;
}
fn main() -> Int {
  var c: Float64 = 0.0;
  var f: Float64 = to_fahrenheit(c);
  if f != 32.0 { return 1; }
  var c2: Float64 = to_celsius(f);
  if c2 != 0.0 { return 2; }
  var t = TempReading{ c: 100.0; f: 212.0; };
  if t.c != 100.0 { return 3; }
  if t.f != 212.0 { return 4; }
  var ct: Celsius = 50.0;
  var ft: Fahrenheit = to_fahrenheit(ct);
  if ft != 122.0 { return 5; }
  return 0;
}
