// B-05 (REL-20261010-1548-bindings-5): alloc/free accounting inside ONE
// confined block. A balanced alloc(8)+free pair repeated 30M times in a
// single `unsafe` block must complete without the guard arena retaining one
// block per pair (pre-fix `xiom_guard_free` was a no-op for arena pointers:
// 16 bytes per pair = 480 MB retained, and the run spent ~25 s in slab
// allocation before the block exited). The loop also writes and reads the
// payload so neither the alloc nor the free can be elided.
module b05_alloc_guard_loop

use xiom.ffi;
use xiom.io;
use xiom.convert;

fn main() -> Int
  requires: true
{
  unsafe {
    var i = 0;
    var acc = 0;
    while i < 30000000 {
      let p = ffi.alloc(8);
      ffi.ptr_write_u64_le(p, 7);
      acc = acc + ffi.ptr_read_u64_le(p);
      ffi.free(p);
      i = i + 1;
    }
    io.println("acc=" + convert.int_to_string(acc));
    io.flush_stdout();
    if acc != 210000000 {
      return 1
    }
    return 0
  }
}
