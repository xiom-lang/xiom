// REL-20261030-1540-bindings (epos): an aggregate binding with a Vec field
// created inside a conditional arm is dropped at a merge/exit block. The drop
// must not emit a field GEP that the arm's alloca does not dominate; the GEP
// is registered in the entry block and the current data pointer is loaded at
// drop time.
module aggregate_arm_drop

use xiom.io;
use xiom.convert;

type Lib = { a: Int; b: Int; c: Int; d: Int; e: Int; data: Vec[UInt8]; }

fn use_lib(l: &Lib) -> Int {
  return l.a + l.data.len();
}

fn probe(flag: Bool) -> Int {
  var total = 0;
  if flag {
    var lib = Lib{ a: 1; b: 2; c: 3; d: 4; e: 5; data: Vec[UInt8].new(); };
    lib.data.push(7);
    lib.data.push(8);
    total = use_lib(&lib);
  } else {
    total = 100;
  }
  return total;
}

fn main() -> Int {
  var i = 0;
  while i < 200 {
    if probe(true) != 3 { return 1; }
    if probe(false) != 100 { return 2; }
    i = i + 1;
  }
  io.println("aggregate arm drop ok");
  return 0;
}
