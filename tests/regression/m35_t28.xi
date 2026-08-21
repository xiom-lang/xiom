// M35-T28: Generic exhaustive -- struct, enum, function, multiple params
fn id[T](x: T) -> T { return x; }
type Box[T] = { val: T; }
type Pair[T, U] = { first: T; second: U; }
enum Opt[T] { Some(v: T), None }
fn opt_has[T](o: Opt[T]) -> Bool { match o { Opt.Some(_) => true, Opt.None => false } }
fn box_val[T](b: Box[T]) -> T { return b.val; }
fn generic_if[T](cond: Bool, a: T, b: T) -> T { if cond { return a; } return b; }
fn generic_while[T](n: Int, v: T) -> Int { var i: Int = 0; while i < n { i = i + 1; } return n; }
fn main() -> Int {
  if id(42) != 42 { return 1; }
  if id("hi") != "hi" { return 2; }
  if id(true) != true { return 3; }
  var bi: Box[Int] = Box{ val: 42; };
  if box_val(bi) != 42 { return 4; }
  var bs: Box[Str] = Box{ val: "a"; };
  if box_val(bs) != "a" { return 5; }
  var oi: Opt[Int] = Opt.Some(99);
  if !opt_has(oi) { return 6; }
  var on: Opt[Int] = Opt.None;
  if opt_has(on) { return 7; }
  var pi: Pair[Int, Str] = Pair{ first: 1; second: "one"; };
  if pi.first != 1 { return 8; }
  if pi.second != "one" { return 9; }
  if generic_if(true, 100, 200) != 100 { return 10; }
  if generic_if(false, "a", "b") != "b" { return 11; }
  if generic_while(5, 99) != 5 { return 12; }
  return 0;
}

