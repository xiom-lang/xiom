// REL-20261010-1554-playground-2 positive control: the explicit integer ->
// Str conversion is legal and concatenates as a Str.
use xiom.io;
use xiom.convert;

fn main() -> Int {
    let s = convert.int_to_string(12) + convert.int_to_string(2);
    io.println(s);
    if s != "122" { return 1; }
    return 0;
}
