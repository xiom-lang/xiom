module b9mod

type IntFrac = { whole: Int; frac: Int; }  // PRIVATE type

pub fn make_frac(w: Int, f: Int) -> IntFrac {
  return IntFrac{ whole: w; frac: f; };
}

pub fn frac_sum(v: IntFrac) -> Int {
  return v.whole + v.frac;
}
