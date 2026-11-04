// REL-20261010-1554-playground-3: a mixed Int/Float expression promotes to
// Float64, so `.to_str()` must print the float text (not the W005 stub).
use xiom.io;

fn main() -> Int {
    let s = (2 + 2.5).to_str();
    io.println(s);
    if s != "4.5" { return 1; }
    return 0;
}
