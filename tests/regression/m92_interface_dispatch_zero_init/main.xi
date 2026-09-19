// R48 lock (playground C17 classes):
// (1) INTERFACE DISPATCH: a generic constrained by an interface called with
//     `&value` must infer the concrete type from the argument. Before R48 the
//     `&`-argument fell into the first-arg "Int" fallback, mono'd
//     `introduce_Int`, and codegen died with
//     `C001: type 'Int' does not implement 'Greetable'`.
// (2) ENUM/MATCH RESULT ZERO-INIT: the match-result slot initializer emitted
//     `store i8* 0` / `store double 0`, which clang rejects ("integer
//     constant must have integer type").
module m92_interface_dispatch_zero_init

use xiom.io;

interface Greetable {
  fn greet() -> Str;
}

type Person = {
  name: Str;
}

fn Person.greet(&self) -> Str {
  return "Hi, " + self.name;
}

fn introduce[T: Greetable](thing: &T) -> Str {
  return thing.greet();
}

enum Direction {
  North,
  South,
  East,
  West,
}

fn heading(dir: Direction) -> Str {
  return match dir {
    North => "north",
    South => "south",
    East => "east",
    West => "west",
  };
}

fn ratio(kind: Direction) -> Float64 {
  return match kind {
    North => 0.5,
    South => 1.5,
    East => 2.5,
    West => 3.5,
  };
}

fn main() -> Int {
  // (1) interface dispatch through a ref argument
  let p = Person{ name: "Alex" };
  if introduce(&p) != "Hi, Alex" { return 1; }

  // (2a) pointer-typed match result + .to_str()
  if heading(Direction.East).to_str() != "east" { return 2; }

  // (2b) double-typed match result
  if ratio(Direction.South) != 1.5 { return 3; }

  io.println(introduce(&p));
  io.println(heading(Direction.West));
  return 0;
}
