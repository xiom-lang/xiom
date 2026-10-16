// m341 fixture (REL-20261015-1538-engine W2.1): a const reached through a
// `pub use` RE-EXPORT chain, read BARE inside a secondary module. The m252
// catalog-body use-skip probe looked only at the target module's own text
// for `pub const`; m341o.reexp declares NOTHING but `pub use m341o.reflect;`
// (and `use m341o.reflect;` in the reader never names the alias in the
// body either), so pre-fix the probe said "no consts", the skip dropped the
// use, the re-exported const never reached the body's surface and the bare
// read was error[T001] undefined variable 'RFL_OK' in catalog body
// [m341o.property2]. The entry-level re-export read and the direct-use read
// inside cons.xi are controls (green pre-fix).
module main

use m341o.property2;
use m341o.reexp;
use m341o.cons;
use xiom.io;

fn main() -> Int {
    // (control) entry-level re-export read -- green pre-fix.
    if RFL_OK != 1 { return 1; }
    // (c) secondary-module re-export read -- pre-fix T001.
    if f2() != 1 { return 2; }
    // (control) direct-use const read inside a secondary module (m317).
    if use_k() != 9 { return 3; }
    return 0;
}
