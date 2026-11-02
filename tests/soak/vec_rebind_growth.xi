// REL-20261102-1605 regression: a Vec local grown by push (realloc) then
// reassigned. The rebind free must read the CURRENT header data, not a
// pointer captured before realloc, or the process dies with heap corruption.
module vec_rebind_growth
use xiom.io;
use xiom.convert;

fn main() -> Int {
  var k = 0;
  var sink = 0;
  while k < 2000 {
    var v = Vec[Int].new();
    var i = 0;
    while i < 100 {
      v.push(i);
      i = i + 1;
    }
    v = Vec[Int].new();
    var j = 0;
    while j < 10 {
      v.push(j);
      j = j + 1;
    }
    sink = sink + v.len();
    k = k + 1;
  }
  io.println("vec_rebind_growth=" + convert.int_to_string(sink));
  return 0;
}
