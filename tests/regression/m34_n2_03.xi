// M34-N2-03: 20-deep addition expression — pushes expression nesting limit
fn main() -> Int {
  var r = (((((((((((((((((((1+2)+3)+4)+5)+6)+7)+8)+9)+10)+11)+12)+13)+14)+15)+16)+17)+18)+19)+20);
  if r == 210 { return 0; }
  return 1;
}
