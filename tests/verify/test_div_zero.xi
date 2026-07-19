module test_div_zero
fn divide(a: Int, b: Int) -> Int
    requires: b != 0
    ensures: result == a / b
{
    return a / b;
}
