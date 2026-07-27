type Age = Int;
type Name = Str;
fn make_person(age: Age, name: Name) -> Int { if age > 0 { return age; } return 0; }
fn main() -> Int { if make_person(25, "a") != 25 { return 1; } return 0; }