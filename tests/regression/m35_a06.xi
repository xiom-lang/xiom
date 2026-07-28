// M35-A06: Merge simulation — count elements <= value across two sorted arrays
fn main() -> Int {
  var a = [1, 4, 7, 10];
  var b = [2, 5, 8, 11, 14];
  var na: Int = 4;
  var nb: Int = 5;
  var total: Int = na + nb;
  if total != 9 { return 1; }
  var i: Int = 0;
  var j: Int = 0;
  var steps: Int = 0;
  while i < na && j < nb {
    if a[i] <= b[j] { i = i + 1; } else { j = j + 1; }
    steps = steps + 1;
  }
  var remaining: Int = (na - i) + (nb - j);
  steps = steps + remaining;
  if steps == total { return 0; }
  return 1;
}
