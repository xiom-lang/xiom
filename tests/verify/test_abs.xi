module test_abs

fn abs(x: Int) -> Int
    requires: x != 0
    ensures: result > 0
{
    if x < 0 {
        return -x;
    }
    return x;
}
