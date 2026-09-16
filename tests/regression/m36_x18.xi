// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X18: Type alias resolution -- complex nested type aliases
type ID = Int;
type Name = Str;
type PlayerID = ID;
type PlayerRecord = { id: PlayerID; name: Name; score: Int; rate: Float64; }
type Team = { members: PlayerRecord; count: Int; }
fn make_player(id: ID, name: Name, score: Int, rate: Float64) -> PlayerRecord {
  return PlayerRecord{ id: id; name: name; score: score; rate: rate; };
}
fn get_score(p: PlayerRecord) -> Int { return p.score; }
fn get_rate(p: PlayerRecord) -> Float64 { return p.rate; }
fn boost_score(p: PlayerRecord, bonus: Int) -> PlayerRecord {
  var r = p;
  r.score = r.score + bonus;
  return r;
}
fn main() -> Int {
  var id: PlayerID = 1;
  var name: Name = "Alice";
  var p = make_player(id, name, 100, 0.75);
  if get_score(p) != 100 { return 1; }
  if get_rate(p) != 0.75 { return 2; }
  var boosted = boost_score(p, 50);
  if get_score(boosted) != 150 { return 3; }
  if p.id != 1 { return 4; }
  if p.name != "Alice" { return 5; }
  var t = Team{ members: p; count: 1; };
  if t.count != 1 { return 6; }
  if t.members.score != 100 { return 7; }
  return 0;
}
