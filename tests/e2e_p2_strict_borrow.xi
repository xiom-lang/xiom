// P2-2 E2E Test: E001 as hard errors with --strict-mode
// Tests that borrow errors (E001) become compilation errors in strict mode.

type Data = { value: Int; }

fn take(s: Data) {
    // just consume the value
}

fn main() -> Int {
    var x = Data { value: 42 };
    take(x);  // move x
    // use of moved value 'x' — should trigger E001
    return x.value;
}
