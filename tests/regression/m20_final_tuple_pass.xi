fn swap(a:Int,b:Int) -> (Int,Int) { return (b,a); }
fn main() -> Int { var p = swap(1,2); if p.0!=2{return 1;} if p.1!=1{return 2;} return 0; }