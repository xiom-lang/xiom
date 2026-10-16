// m341 fixture entry (import-order twin B, REL-20261015-1538-engine W2.2):
// `use m341o.reflect;` BEFORE `use m341o.cons;` -- the order that failed
// pre-m317. Must stay green.
module order_b

use m341o.reflect;
use m341o.cons;
use xiom.io;

fn main() -> Int {
    if use_k() != 9 { return 1; }
    if K != 9 { return 2; }
    return 0;
}
