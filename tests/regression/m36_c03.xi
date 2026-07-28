// M36-C03: Every match pattern with every enum shape — unit variants and single-payload variants, nested matches, guards
enum Status1 { Pending }
enum Status2 { Off, On }
enum Status3 { Low, Mid, High }
fn match_small(e: Status1) -> Int { match e { Status1.Pending => 1 } }
fn match_pair(e: Status2) -> Int { match e { Status2.Off => 0, Status2.On => 1 } }
fn match_triple(e: Status3) -> Int { match e { Status3.Low => 1, Status3.Mid => 2, Status3.High => 3 } }
fn main() -> Int {
  if match_small(Status1.Pending) != 1 { return 1; }
  if match_pair(Status2.Off) != 0 { return 2; }
  if match_pair(Status2.On) != 1 { return 3; }
  if match_triple(Status3.Low) != 1 { return 4; }
  if match_triple(Status3.Mid) != 2 { return 5; }
  if match_triple(Status3.High) != 3 { return 6; }
  var x: Int = 0;
  match true { true => { x = 1; } false => { x = -1; } }
  if x != 1 { return 7; }
  var o: Option[Int] = Some(42);
  match o { Some(v) => { if v != 42 { return 8; } } None => { return 9; } }
  return 0;
}
