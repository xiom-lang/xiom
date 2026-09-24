// Stage 6 perf fixture: the fmt-peek closure shape.
//
// The first `.to_str()` compile used to pull the whole `xiom.fmt` catalog
// closure through `collect_external_decls`'s peek (sweep p50 3.9 -> 7.9 s
// for this shape). The documented fix (a reachable-function-only peek) is
// deferred to the Stage 6 catalog-index work; this fixture locks the
// emitted IR (deterministic) and a wall-time ceiling so the regression
// cannot grow unnoticed.
module perf_fmt_peek;

fn main() -> Int {
  var n = 42;
  let s = n.to_str();
  if s.len() == 0 { return 1; }
  var f = 1.5;
  let g = f.to_str();
  if g.len() == 0 { return 2; }
  return 0;
}
