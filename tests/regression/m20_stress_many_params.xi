fn f(a:Int,b:Int,c:Int,d:Int,e:Int) -> Int { return a+b+c+d+e; }
fn main() -> Int { if f(1,2,3,4,5) != 15 { return 1; } return 0; }