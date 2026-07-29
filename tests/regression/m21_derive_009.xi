module m21_derive_009
type Span = {
    line: Int;
    col: Int;
  } derive[Eq, Clone, Hash, Ord, Display]

  pub fn run() -> Int {
    var s: Span = { line: 10; col: 5; };
    if s.line == 10 && s.col == 5 { return 0; }
    return 1;
  }
use m21_derive_009.run;
fn main() -> Int { return run(); }
