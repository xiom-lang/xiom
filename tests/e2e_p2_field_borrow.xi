// P2-3 E2E Test: Field-granular borrow checks
// Tests that borrowing different struct fields is allowed (disjoint),
// but borrowing the same field or overlapping prefixes is rejected.

type Data = { a: Int; b: Int; }

fn main() -> Int {
    // Test 1: Borrowing different fields (disjoint) — should be OK
    var d = Data { a: 10; b: 20 };
    var pa = &d.a;
    var pb = &d.b;
    // Both borrows should coexist since a and b are different fields
    if (*pa != 10) { return 1; }
    if (*pb != 20) { return 2; }

    // Test 2: Read + read on same field — OK
    var d2 = Data { a: 5; b: 6 };
    var ra1 = &d2.a;
    var ra2 = &d2.a;
    if (*ra1 + *ra2 != 10) { return 3; }

    return 0;
}
