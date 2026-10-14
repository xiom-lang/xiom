// m309 (REL-20261011-2320-bindings part 3): reversed import order -- pre-fix
// this made BETA (the first use) the T001 and ALPHA resolve, proving the
// resolution followed registration order, not the import set.
use xiom.m309seg.fam.beta;
use xiom.m309seg.fam.alpha;

fn main() -> Int {
    if ALPHA != 11 { return 1; }
    if BETA != 22 { return 2; }
    if alpha.ALPHA != 11 { return 3; }
    if beta.BETA != 22 { return 4; }
    return 0;
}
