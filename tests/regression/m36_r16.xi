// M36-R16: reserved word misuse -- contextual keyword test, function with match
fn main() -> Int { var x = 0; match x { 0 => { return 0; } _ => { return 1; } } }
