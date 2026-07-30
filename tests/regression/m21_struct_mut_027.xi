module m21_struct_mut_027
type Item = { id: Int; weight: Float64; }
fn main() -> Int {
  var items: Vec[Item] = [{ id: 1; weight: 2.5; }, { id: 2; weight: 3.0; }];
  items[0].weight = 4.5;
  items[1].id = 99;
  if items[0].weight == 4.5 && items[1].id == 99 { return 0; }
  return 1;
  return 1;
}
