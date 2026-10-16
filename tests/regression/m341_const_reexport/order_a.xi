// m341 fixture entry (import-order twin A, REL-20261015-1538-engine W2.2):
// `use m341o.cons;` BEFORE `use m341o.reflect;`. Both orders were red on
// 0.64.4 and are green since m317; this twin pins order A.
module order_a

use m341o.cons;
use m341o.reflect;
use xiom.io;

fn main() -> Int {
    if use_k() != 9 { return 1; }
    if K != 9 { return 2; }
    return 0;
}
