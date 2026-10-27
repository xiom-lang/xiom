// REL-20261026-2142 / t1 RSS: Vec[UInt8].with_len(n) must allocate and
// zero n bytes, not n*8. Before the coordinated slot-stride change this
// 4 MiB buffer reserved and touched 32 MiB (the t1 Peak Memory gap).
module vec_uint8_packed_slots

use xiom.io;
use xiom.convert;
use xiom.convert.parse;

fn main() -> Int {
  let n = 4 * 1024 * 1024;
  let v = Vec[UInt8].with_len(n);
  if v.len() != n { return 1; }
  if v[0] != 0 as UInt8 { return 2; }
  if v[n - 1] != 0 as UInt8 { return 3; }
  io.println("vec uint8 packed slots ok len=" + convert.int_to_string(v.len()));
  return 0;
}
