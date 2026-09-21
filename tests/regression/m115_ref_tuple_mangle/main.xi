// m115 (R60, stdlib p_ref_tuple_mangle): a reference-typed tuple element
// mangled into the struct name (%struct.Tuple__&Vec__Vec -> clang "expected
// '=' after name"). The shared tuple element namer now strips reference
// markers and names container-ctor elements by their container base, so
// `(seed, Vec[UInt8].new())` matches the declared Tuple__Vec__Vec.
module m115.main

fn pair(seed: &Vec[UInt8]) -> (Vec[UInt8], Vec[UInt8]) {
  return (seed, Vec[UInt8].new());
}

fn main() -> Int {
  var v: Vec[UInt8] = Vec[UInt8].new();
  let p = pair(&v);
  return 0;
}
