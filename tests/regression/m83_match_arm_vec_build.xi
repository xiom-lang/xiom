// m83: R29 lock -- building a Vec inside a match arm over a
// Result[Vec[...]] payload must compile (clang) and behave. Pre-fix, the
// block compiler stored every expression statement's value into the match
// result slot, so `out.push(...)` inside the arm's while body emitted
// `store %struct.Option <Vec>` (invalid IR).
module m83_match_arm_vec_build

fn pick(flag: Bool) -> Result[Vec[Int], Str] {
  if flag {
    var v = Vec.new[Int]();
    v.push(1);
    v.push(2);
    return Ok(v);
  }
  return Err("no");
}

fn build(flag: Bool) -> Option[Vec[Int]] {
  let r = pick(flag);
  match r {
    Ok(b) => {
      var out = Vec[Int].new();
      var i = 0;
      while i + 1 < b.len() {
        let hi = b[i] as Int;
        let lo = b[i + 1] as Int;
        out.push((hi << 8) | lo);
        i = i + 2;
      };
      return Some(out);
    },
    Err(_) => { return None; },
  }
}

fn main() -> Int {
  let a = build(true);
  if !a.is_some { return 1; }
  let v = a.value;
  if v.len() != 1 { return 2; }
  if v[0] != 258 { return 3; }

  let b = build(false);
  if b.is_some { return 4; }

  return 0;
}
