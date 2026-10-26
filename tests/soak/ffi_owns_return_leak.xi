// m375 / REL-20261026-1158-compiler: `#[owns_return]` must reclaim transient
// Vecs whose pointer is exported to FFI inside `unsafe`, including a local
// that is rebound to a fresh Vec on a retry branch (font_shape's buf/slots,
// font_glyph_raster's pix/slots). Before the fix, the pointer export marked
// `buf` escaping and the rebind marked it escaping again, so every call leaked
// its transient buffer.
module ffi_owns_return_leak

use xiom.io;
use xiom.convert;
use xiom.ffi.c;

type Frame = {
  ok: Bool;
  bytes: Vec[UInt8];
}

#[owns_return]
fn make_frame(i: Int, n: Int) -> Frame {
  var buf = Vec[UInt8].with_len(n);
  var slots = Vec[Int32].with_len(8);
  var rc = 0;
  unsafe {
    rc = c_memset(buf.as_mut_ptr() as Int, 0, n);
  }
  // The retry branch is never taken here, but it must not make `buf` escape.
  if rc == 0 - 3 {
    buf = Vec[UInt8].with_len(n / 2);
    unsafe {
      rc = c_memset(buf.as_mut_ptr() as Int, 0, n / 2);
    }
  }
  slots[0] = i as Int32;
  let count = (slots[0] as Int) % (n + 1);
  var out = Vec[UInt8].with_len(count);
  var k = 0;
  while k < count {
    out[k] = buf[k];
    k = k + 1;
  }
  return Frame{ ok: true; bytes: out };
}

fn main() -> Int {
  let n = 300000;
  let size = 4096;
  var i = 0;
  var sink = 0;
  while i < n {
    let f = make_frame(i, size);
    sink = sink + f.bytes.len();
    i = i + 1;
  }
  println("ffi owns_return leak probe done sink=" + int_to_string(sink));
  return 0;
}
