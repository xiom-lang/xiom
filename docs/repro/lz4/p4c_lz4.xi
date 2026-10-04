// docs/repro/lz4/p4c_lz4.xi -- P4c reproducer for the lz4 empty block-compress.
// Regression (v0.62.2..v0.62.4, from m165's 2^32 Vec growth ceiling):
// compile with clang -O2 emits a peephole-opt miscompile of
// `compress.lz4._lz4_write_seq`; the direct block-compress result reads
// back as len 0 and the probe exits 5. Fixed by m190 (ceiling 2^32-1).
// Expected: rc 0. Root cause: docs/COMPILER_BUGS.md 2026-10-04 entry.
module smoke_compress_lz4_snappy
use xiom.compress.lz4;
use xiom.compress.snappy;
use xiom.io;

fn bytes_equal(a: Vec[UInt8], b: Vec[UInt8]) -> Bool {
  if a.len() != b.len() {
    return false;
  }
  var i = 0;
  while i < a.len() {
    var x = a[i];
    var y = b[i];
    if x != y {
      return false;
    }
    i = i + 1;
  }
  return true;
}

fn main() -> Int {
  var small = Vec[UInt8].new();
  var i = 0;
  while i < 32 {
    small.push((65 + (i % 7)) as UInt8);
    i = i + 1;
  }
  var j = 0;
  while j < 16 {
    small.push(97);
    j = j + 1;
  }
  if small.len() != 48 { return 20; }

  var lz = lz4.lz4_compress(small);
  if lz.len() == 0 { return 1; }
  var dl = lz4.lz4_decompress(lz);
  match dl {
    Ok(v) => { if !bytes_equal(v, small) { return 2; } }
    Err(e) => { return 3; }
  }
  var bound = lz4.lz4_bound(48);
  if bound <= 0 { return 4; }

  var blk = lz4.lz4_compress_block(small);
  if blk.len() == 0 { return 5; }
  var dblk = lz4.lz4_decompress_block(blk);
  match dblk {
    Ok(v) => { return 0; }
    Err(e) => { return 7; }
  }
}
