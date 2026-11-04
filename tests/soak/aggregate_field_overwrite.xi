// REL-2117: a struct literal built in this function with a fresh-return Vec
// field is uniquely owned. Each `d.v = mk_vec(..)` must free the REPLACED
// field data, and the scope-exit aggregate drop frees the final one. Without
// the field overwrite drop this leaks ~8,192 Ints per iteration (~1.9 GB at
// 30k iterations).
module aggregate_field_overwrite

use xiom.io;
use xiom.convert;

type Holder = {
  v: Vec[Int];
  n: Int;
}

#[owns_return]
fn mk_vec(n: Int) -> Vec[Int] {
  var b = Vec[Int].new();
  var i = 0;
  while i < n {
    b.push(i);
    i = i + 1;
  }
  return b;
}

fn run(iters: Int) -> Int {
  var d = Holder{ v: mk_vec(8192); n: 0 };
  var sink = 0;
  var i = 0;
  while i < iters {
    d.v = mk_vec(8192);
    sink = sink + 1;
    i = i + 1;
  }
  return sink;
}

fn main() -> Int {
  var a = run(10000);
  var b = run(20000);
  io.println("aggregate_field_overwrite=" + convert.int_to_string(a + b));
  return 0;
}
