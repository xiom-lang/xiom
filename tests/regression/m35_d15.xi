// M35-D15: LRU cache -- recency via access counters
fn main() -> Int {
  var t0: Int = 1;
  var t1: Int = 3;
  var t2: Int = 5;
  var oldest: Int = t0;
  if t1 < oldest { oldest = t1; }
  if t2 < oldest { oldest = t2; }
  if oldest != 1 { return 1; }
  t0 = 10;
  if t0 != 10 { return 2; }
  return 0;
}
