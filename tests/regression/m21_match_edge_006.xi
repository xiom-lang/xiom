module m21_match_edge_006
enum Animal {
    Dog(name: Str, age: Int),
    Cat(name: Str, color: Str),
    Bird(species: Str),
  }

  pub fn run() -> Int {
    var a = Animal.Dog("Rex", 5);
    match a {
      Animal.Dog(_, age) => if age == 5 { return 0; },
      Animal.Cat(_, _) => return 1,
      Animal.Bird(_) => return 1,
    }
  }
use m21_match_edge_006.run;
fn main() -> Int { return run(); }
