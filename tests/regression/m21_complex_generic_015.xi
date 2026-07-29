module m21_complex_generic_015
pub fn swap_pair[A, B](a: A, b: B) -> { fst: B; snd: A; } {
    return { fst: b; snd: a; };
  }

  pub fn run() -> Int {
    var p = swap_pair[Int, Str](1, "one");
    if p.fst == "one" && p.snd == 1 { return 0; }
    return 1;
  }
use m21_complex_generic_015.run;
fn main() -> Int { return run(); }
