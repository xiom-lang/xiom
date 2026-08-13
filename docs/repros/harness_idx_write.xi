use repro_idx_write;

fn main() -> Int {
  var key = Vec[UInt8].new();
  var i = 0;
  while i < 16 {
    key.push(i as UInt8);
    i = i + 1;
  }
  var s = make_sched(&key);
  var r = check_sched(&s);
  if r != 0 { return r; }
  return 0;
}
