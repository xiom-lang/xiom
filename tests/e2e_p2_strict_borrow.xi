// P2-2 E2E Test: E001 as hard errors with --strict-mode
// Tests that borrow errors (E001) become compilation errors in strict mode.

type Data = { value: Int; }

fn take(s: Data) {
    var _ = s;
}

fn main() -> Int {
    var x = Data { value: 42 };
    take(x);  // move x
    // x is now moved — using it triggers E001
    var _ = x;  // E001: use of moved value
    return 0;
}
