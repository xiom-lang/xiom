// P1-4 E2E Test: Contract collection methods
// Tests that is_sorted(), contains() are recognized by the checker
// and compile correctly (codegen has xiom_is_sorted/xiom_contains stubs).

fn main() -> Int {
    var arr = [1, 2, 3, 4, 5];

    // Test 1: is_sorted() -- recognized by checker, compiles via xiom_is_sorted
    var sorted = arr.is_sorted();

    // Test 2: contains() -- recognized by checker
    var has_three = arr.contains(3);

    // Test 3: contains() with missing element
    var has_nine = arr.contains(9);

    return 0;
}
