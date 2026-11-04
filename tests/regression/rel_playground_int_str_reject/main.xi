// REL-20261010-1554-playground-2: Int + Str is an implicit conversion and
// must be a checker error (SPEC 9: conversions are explicit).
use xiom.io;

fn main() -> Int {
    io.println(12 + 2.to_string());
    return 0;
}
