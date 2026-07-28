// M36-C28: Combined mega test 1 — structs, enums, match, recursion, loops, Option
type Stats = { sum: Int; count: Int; } derive[Eq]
fn Stats.new() -> Stats { return Stats{ sum: 0; count: 0; }; }
fn Stats.add(self, n: Int) -> Stats { var r = self; r.sum = r.sum + n; r.count = r.count + 1; return r; }
fn Stats.avg(self) -> Int {
  if self.count == 0 { return 0; }
  return self.sum / self.count;
}
fn factorial_r(n: Int) -> Int { if n <= 1 { return 1; } return n * factorial_r(n - 1); }
fn sum_range(n: Int) -> Int { var s = 0; var i = 0; while i <= n { s = s + i; i = i + 1; } return s; }
fn classify(n: Int) -> Int {
  if n < 0 { return -1; }
  if n == 0 { return 0; }
  return 1;
}
fn main() -> Int {
  var s = Stats.new();
  s = s.add(10);
  s = s.add(20);
  s = s.add(30);
  if s.sum != 60 { return 1; }
  if s.count != 3 { return 2; }
  var avg_v = s.avg();
  if avg_v != 20 { return 3; }
  if factorial_r(5) != 120 { return 4; }
  if sum_range(10) != 55 { return 5; }
  if classify(5) != 1 { return 6; }
  if classify(0) != 0 { return 7; }
  if classify(-5) != -1 { return 8; }
  var opt: Option[Int] = Some(42);
  match opt { Some(v) => { if v != 42 { return 9; } } None => { return 10; } }
  var optn: Option[Int] = None;
  match optn { Some(_) => { return 11; } None => {} }
  return 0;
}
