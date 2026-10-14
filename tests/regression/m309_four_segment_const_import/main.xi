// m309 (REL-20261011-2320-bindings part 3): unqualified cross-module const
// imports with 4-segment module paths must resolve EVERY imported module's
// bare constants, not only the last-use'd one (LIFO worklist registration).
// Pre-fix: T001 undefined variable 'ALPHA' (this order); flips with the
// import order (see main_flip.xi). Module-qualified access is the control.
use xiom.m309seg.fam.alpha;
use xiom.m309seg.fam.beta;

fn main() -> Int {
    if ALPHA != 11 { return 1; }
    if BETA != 22 { return 2; }
    if alpha.ALPHA != 11 { return 3; }
    if beta.BETA != 22 { return 4; }
    return 0;
}
