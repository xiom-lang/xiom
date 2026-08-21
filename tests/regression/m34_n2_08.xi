// M34-N2-08: 10-level nested if-blocks -- pushes block nesting limit
fn main() -> Int {
  var x = 0;
  if 1 == 1 {
  if 1 == 1 {
  if 1 == 1 {
  if 1 == 1 {
  if 1 == 1 {
  if 1 == 1 {
  if 1 == 1 {
  if 1 == 1 {
  if 1 == 1 {
  if 1 == 1 {
    x = 42;
  } } } } } } } } } }
  if x == 42 { return 0; }
  return 1;
}
