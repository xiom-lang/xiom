// m341 fixture module: reads the re-exported const BARE. Pre-fix this body
// was error[T001] undefined variable 'RFL_OK' in the catalog-body check.
module m341o.property2

use m341o.reexp;

pub fn f2() -> Int { return RFL_OK; }
