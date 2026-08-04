// P2-6 E2E Test: Multi-type turbofish
// Tests that ::<Type1, Type2> syntax parses correctly.

fn main() -> Int {
    // Test 1: sizeof with single type (regression check)
    var s1 = sizeof::<Int>();

    // Test 2: sizeof::<Int, Str, Bool> — multi-type turbofish parsing
    // (sizeof only uses the first type arg, but parsing multiple works)
    var s2 = sizeof::<Int, Str>();

    // Test 3: align_of with multi-type
    var a1 = align_of::<Int, Str>();

    // Test 4: type_id with single type (regression)
    var t1 = type_id::<Int>();

    // All compile checks passed
    return 0;
}
