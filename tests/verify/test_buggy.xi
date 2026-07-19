// BUG: ensures says result > 0 but -x can be negative when x is positive.
// The negated ensures: (not (> result 0)) should be SAT for x = 5.
fn buggy_abs(x: Int) -> Int
    requires: x != 0
    ensures: result > 0
{
    // Bug: should return -x only when x < 0
    return -x;
}
