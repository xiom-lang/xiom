// M36-R12: wrong type name -- type alias defining custom types, valid usage
type AliasInt = Int;

fn main() -> Int { var x: AliasInt = 42; return 0; }
