module m37_structural_eq
// BUG 24 residual fix: `==`/`!=` on same-type structs WITHOUT a derived eq
// must compare ALL fields structurally. The old fallback compared only
// field 0 (a multi-field struct compared its first field — silent
// miscompare).

type Pt = { x: Int; y: Int; }

fn main() -> Int {
  var a = Pt { x: 3; y: 4; };
  var b = Pt { x: 3; y: 4; };
  var c = Pt { x: 3; y: 5; };
  var d = Pt { x: 9; y: 4; };
  if a == b { } else { return 1; }
  if a != c { } else { return 2; }
  if a == c { return 3; }
  if a == d { return 4; }
  if a != b { return 5; }
  return 0;
}
