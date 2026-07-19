// G-24: Float32 ARM ABI — verify Float32 operations are correct on ARM targets
// ARM uses IEEE 754 single-precision same as x86; the IR must use `float` type.
// Cross-compiled for aarch64-linux-gnu via clang.

fn main() -> Int {
  // Basic Float32 arithmetic
  let a: Float32 = 1.5;
  let b: Float32 = 2.5;
  let c: Float32 = a + b;
  if c > 3.9 { return 0; }
  return 1;
}
