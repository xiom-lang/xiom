// m179 lock (release hold, stdlib-perf2): `#[unsafe_direct]` sync bodies.
//
// xiom.sync's `Arc.new[T]` sizes the control block with
// `size_of[ArcInner[T]]()`. The generic-call parser reduces the nested type
// arg to its base name ("ArcInner"), and the size intrinsic only consulted
// the AST index/arg sources -- so the monomorphised `Arc.new_Int` fell to
// the 8-byte scalar fallback: malloc(8) for a 16-byte ArcInner, then a
// 16-byte struct store into it (heap overflow). `strong_count()` read
// garbage and reported != 1 immediately after construction.
//
// Needs a stdlib checkout with the `xiom/sync/sync.xi` unsafe_direct
// annotations (stdlib-perf2+); older pins without the annotation pass
// vacuously.
module m179_arc_strong_count

use xiom.sync;

fn main() -> Int {
  var a = sync.Arc.new(42);
  if a.strong_count() != 1 { return 1; }
  if a.get() != 42 { return 2; }
  return 0;
}
