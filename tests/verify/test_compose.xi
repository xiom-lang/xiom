module test_compose

fn square(x: Int) -> Int
    requires: x >= 0
    ensures: result >= 0
{
    return x * x;
}

fn use_square(a: Int) -> Int
    requires: a >= 0
    ensures: result >= 0
{
    return square(a);
}
