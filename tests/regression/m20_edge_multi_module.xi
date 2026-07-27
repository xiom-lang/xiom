use stdlib.xiom.string;
use stdlib.xiom.io;
fn main() -> Int { var s = "hi"; if string.str_len(s) != 2 { return 1; } return 0; }