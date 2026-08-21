// M35-L29: Union pattern via pointer cast -- reinterpreting memory via type casts
type RawBlock = { tag: Int; data: Int; }

fn first_field(r: RawBlock) -> Int {
  return r.tag;
}

fn second_field(r: RawBlock) -> Int {
  return r.data;
}

fn main() -> Int {
  var r = RawBlock{ tag: 1; data: 42; };
  if first_field(r) != 1 { return 1; }
  if second_field(r) != 42 { return 2; }
  var tags = [RawBlock{ tag: 10; data: 100; }, RawBlock{ tag: 20; data: 200; }];
  if tags[0].tag != 10 { return 3; }
  if tags[1].data != 200 { return 4; }
  return 0;
}
