// m312 (REL-20261013-1423 thread readback): a `*T` struct FIELD bound to
// a local (`let q = c.buf;`) or used as a method receiver field (`buf` in
// `Cell.at8`) must lower `+ Int` to POINTER arithmetic. The i8* LLVM slot
// alone is ambiguous with Str, so pre-fix `q + 8` became
// xiom_str_concat(q, "8") and the read went through a decimal-digit
// string -- JoinHandle.join's `result_buf + 8` returned garbage while the
// spawned worker had written the correct result.
module m312_ptr_field_arith

use xiom.ffi;

type Cell = { tag: Int; buf: *UInt8; }

fn Cell.at8(self) -> Int
  requires: true
{
  unsafe {
    return ffi.ptr_read_u64_le(buf + 8);
  }
}

fn main() -> Int
  requires: true
{
  unsafe {
    let p = ffi.alloc(16);
    let c = Cell{ tag: 1; buf: p };
    ffi.ptr_write_u64_le(c.buf + 8, 1234567);
    let q = c.buf;
    if ffi.ptr_read_u64_le(q + 8) != 1234567 {
      return 1;
    };
    if c.at8() != 1234567 {
      return 2;
    };
    if ffi.ptr_read_u64_le(c.buf + 8) != 1234567 {
      return 3;
    };
    return 0;
  }
}
