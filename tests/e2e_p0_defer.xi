// P0-2 E2E Test: defer scope-exit execution
// Tests that defer blocks execute at scope exit (before return), not inline.

var defer_log = 0;  // global counter for defer verification

fn test_defer_basic() -> Int {
    defer { defer_log = 1; }
    // At this point defer_log should still be 0 (defer hasn't run yet)
    var before = defer_log;
    if before != 0 { return 1; }  // defer should NOT have run yet
    return 0;
}

fn test_defer_lifo() -> Int {
    // Multiple defers should execute in LIFO order
    defer { defer_log = defer_log + 1; }   // runs third
    defer { defer_log = defer_log * 2; }   // runs second (0*2=0)
    defer { defer_log = 42; }              // runs first
    return 0;
}

fn test_defer_with_return() -> Int {
    defer { defer_log = 99; }
    // Early return — defer should still execute
    if defer_log == 0 { return 0; }
    return 1;
}

fn main() -> Int {
    // Test 1: Basic defer
    defer_log = 0;
    var r1 = test_defer_basic();
    if r1 != 0 { return 1; }
    // After return, defer should have executed
    if defer_log != 1 { return 2; }  

    // Test 2: LIFO order (42 then *2 then +1 = 85)
    defer_log = 0;
    var r2 = test_defer_lifo();  
    if r2 != 0 { return 3; }
    if defer_log != 85 { return 4; }  // (0→42 → *2=84 → +1=85)

    // Test 3: Defer with early return
    defer_log = 0;
    var r3 = test_defer_with_return();
    if r3 != 0 { return 5; }
    if defer_log != 99 { return 6; }

    return 0;
}
