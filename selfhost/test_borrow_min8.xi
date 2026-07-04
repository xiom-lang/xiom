fn xiom_str_len(src: Int) -> Int;
fn xiom_char_at(src: Int, pos: Int) -> Int;

fn test(src: Int) -> Int {
  var len = xiom_str_len(&src);
  var pos = 0;
  while (pos + 2 < len) {
    var c0 = xiom_char_at(&src, &pos);
    if c0 == 102 {
      var ws_c = xiom_char_at(&src, &pos);
      var body_start = pos;
    }
  }
  return 0;
}
fn main() -> Int { return 0; }
