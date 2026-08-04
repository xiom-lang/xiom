// P1-3 E2E Test: Float literal patterns in match
// Tests match with float literal patterns.

fn main() -> Int {
    // Test 1: Match on float literal
    var x = 3.14;
    var matched = false;
    match x {
        3.14 => { matched = true; }
        _ => {}
    }
    if !matched { return 1; }

    // Test 2: Match with multiple float arms
    var y = 2.5;
    var result = 0;
    match y {
        1.0 => { result = 1; }
        2.5 => { result = 2; }
        _ => { result = 99; }
    }
    if result != 2 { return 2; }

    // Test 3: Float pattern with guard
    var z = 0.0;
    var found = false;
    match z {
        0.0 if true => { found = true; }
        _ => {}
    }
    if !found { return 3; }

    // Test 4: Float OR-pattern
    var w = 4.0;
    var hit = 0;
    match w {
        1.0 | 2.0 | 4.0 => { hit = 4; }
        _ => {}
    }
    if hit != 4 { return 4; }

    return 0;
}
