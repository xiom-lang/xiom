module m21_string_007
type User = { name: Str; email: Str; }

  pub fn run() -> Int {
    var u: User = { name: "Alice"; email: "a@x.com"; };
    if u.name == "Alice" && u.email == "a@x.com" { return 0; }
    return 1;
  }
use m21_string_007.run;
fn main() -> Int { return run(); }
