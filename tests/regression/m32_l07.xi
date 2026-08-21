// M32-L07: If-elif-else chain (5 branches) -- temperature classifier
fn main() -> Int {
  var temp: Int = 25;
  var zone: Int = 0;
  if temp <= 0 {
    zone = 1;
  } elif temp <= 15 {
    zone = 2;
  } elif temp <= 30 {
    zone = 3;
  } elif temp <= 45 {
    zone = 4;
  } else {
    zone = 5;
  }
  if zone == 3 { return 0; }
  return 1;
}
