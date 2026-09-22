use xiom.io;
use xiom.iter;

fn min_val(scores: &Vec[Int]) -> Int
  requires: scores.len() > 0;
{
  let m = scores[0];
  for __i in range(0, scores.len()) {
    let s = scores[__i];
    if s < m { m = s; } }
  return m;
}

fn max_val(scores: &Vec[Int]) -> Int
  requires: scores.len() > 0;
{
  let m = scores[0];
  for __i in range(0, scores.len()) {
    let s = scores[__i];
    if s > m { m = s; } }
  return m;
}

fn range(scores: &Vec[Int]) -> Int
  requires: scores.len() > 0;
{
  return max_val(scores) - min_val(scores);
}

fn main() -> Int {
  let data = Vec[Int].new();
  data.push(10);
  data.push(25);
  data.push(5);
  data.push(30);
  return range(&data);
}