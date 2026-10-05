// m191 lock (benchmark relay): a BARE call to a duplicated stdlib leaf must
// bind the CHECKER's resolution, not codegen's registration-order bare slot.
// `lz4_compress` exists in two modules: xiom.compress.lz4 (-> Vec[UInt8],
// the imported module the checker resolves) and xiom.compress (-> Result,
// the umbrella wrapper that codegen's bare slot used to bind). Pre-fix the
// frame length read aggregate garbage (2740398262480 in the probe) and the
// lz4 smoke child exited -1.
module m191_lz4_bare_duplicate_leaf

use xiom.io;
use xiom.string.slice;
use xiom.compress.lz4;

fn main() -> Int {
  var input = str_bytes("m191 lz4 duplicate-leaf bare-call lock payload");
  var frame = lz4_compress(&input);
  if frame.len() == 0 || frame.len() > input.len() + 128 {
    io.println("m191: bad frame length");
    return 1;
  }
  match lz4_decompress(&frame) {
    Ok(out) => {
      if out.len() != input.len() {
        io.println("m191: round-trip length mismatch");
        return 2;
      }
    }
    Err(e) => {
      io.println("m191: decompress failed: " + e);
      return 3;
    }
  }
  io.println("M191 OK");
  return 0;
}
