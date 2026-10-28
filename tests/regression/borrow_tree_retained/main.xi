// REL-20261028-1801/1805 lock: `_tree_from_lengths(lens: &Vec[Int])` stores
// the BORROWED lens header in the returned HuffmanTree. The v0.65.2 blanket
// "borrowed arg is non-retaining" rule freed `lens` in huffman_build while
// the returned tree still pointed at it -> llvm.trap/SIGILL (0xC000001D) in
// huffman_encode. Only scalar/void-returning callees may be non-retaining.
module borrow_tree_retained

use xiom.io;
use xiom.compress.huffman;

fn main() -> Int {
  var b = Vec[UInt8].new();
  var i = 0;
  while i < 5000 {
    b.push((65 + ((i * 7 + 3) % 26)) as UInt8);
    i = i + 1;
  }
  var h = huffman.huffman_compress(&b);
  match huffman.huffman_decompress(&h) {
    Ok(out) => {
      if out.len() != 5000 { return 1; }
      if out[0] != b[0] { return 2; }
      if out[4999] != b[4999] { return 3; }
    },
    Err(_) => { return 4; },
  }
  io.println("borrow tree retained ok");
  return 0;
}
