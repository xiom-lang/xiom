fn axiom_str_len(src: Int) -> Int;
fn axiom_char_at(src: Int, pos: Int) -> Int;

fn test(src: Int) -> Int {
  var len = axiom_str_len(&src);
  var pos = 0;
  while (pos + 2 < len) {
    var c0 = axiom_char_at(&src, &pos);
    if c0 == 102 {
      var ws_c = axiom_char_at(&src, &pos);
      var body_start = pos;
    }
  }
  return 0;
}
fn main() -> Int { return 0; }
