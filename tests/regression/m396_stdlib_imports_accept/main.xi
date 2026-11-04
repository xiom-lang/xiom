// m396 regression lock: importing stdlib modules whose bodies contain
// deferred `let x: T;` initializations (FFI/OS) and receiver-field stores
// must produce ZERO T001 immutable-binding diagnostics.
use xiom.collections;
use xiom.io;
use xiom.math;
use xiom.os.mmap;
use xiom.os.sync_io;

fn main() -> Int {
  return 0;
}
