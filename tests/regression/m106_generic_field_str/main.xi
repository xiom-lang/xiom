// R52 lock (playground L5-20/L5-31): Str fields of a CONCRETE generic struct
// must load as i8* and `.to_str()` must keep the Str representation -- was the
// erased i64 field whose address got printed as a number.
use xiom.io;

type Pair[T, U] = {
  first: T;
  second: U;
}

type Box[T] = {
  value: T;
}

fn Box.get_value[T](self) -> T {
  return self.value;
}

fn main() -> Int {
  let greeting: Pair[Str, Str] = Pair[Str, Str]{ first: "Hello", second: "World" };
  if greeting.first.to_str() != "Hello" { return 1; }
  if greeting.second.to_str() != "World" { return 2; }

  let str_box: Box[Str] = Box[Str]{ value: "XIOM" };
  if str_box.get_value().to_str() != "XIOM" { return 3; }

  let int_box: Box[Int] = Box[Int]{ value: 100 };
  if int_box.get_value().to_str() != "100" { return 4; }

  io.println("ok");
  return 0;
}
