use repro_tuple_destructure;

fn main() -> Int {
  // THE AES-GCM SHAPE: let (vec, int) = fn()
  let (v, nr) = key_expansion();
  var r = use_key(&v, nr);
  if r != 0 { return r; }
  return 0;
}
