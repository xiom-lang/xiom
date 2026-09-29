// m158 (Stage 6 W007) negative: float self-comparison (NaN semantics),
// distinct operands, calls, and struct self-comparison must stay silent.
module m158_w007_guard;

type Q = { n: Float64; }

fn side() -> Int { return 7; }

fn main() -> Int {
  let x = 5;
  let y = 5;
  let f = 1.5;
  let a = f == f;
  let b = x == y;
  let c = side() == side();
  let p = Q { n: 1.5 };
  let d = p == p;
  if !a { return 1; }
  if !b { return 2; }
  if !c { return 3; }
  if !d { return 4; }
  return 0;
}
