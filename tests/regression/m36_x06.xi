// M36-X06: Large match with 50+ arms — exhaustive enum dispatch
enum State {
  S00, S01, S02, S03, S04, S05, S06, S07, S08, S09,
  S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
  S20, S21, S22, S23, S24, S25, S26, S27, S28, S29,
  S30, S31, S32, S33, S34, S35, S36, S37, S38, S39,
  S40, S41, S42, S43, S44, S45, S46, S47, S48, S49,
  S50, S51, S52, S53, S54
}
fn state_to_int(s: State) -> Int {
  match s {
    S00 => 0, S01 => 1, S02 => 2, S03 => 3, S04 => 4,
    S05 => 5, S06 => 6, S07 => 7, S08 => 8, S09 => 9,
    S10 => 10, S11 => 11, S12 => 12, S13 => 13, S14 => 14,
    S15 => 15, S16 => 16, S17 => 17, S18 => 18, S19 => 19,
    S20 => 20, S21 => 21, S22 => 22, S23 => 23, S24 => 24,
    S25 => 25, S26 => 26, S27 => 27, S28 => 28, S29 => 29,
    S30 => 30, S31 => 31, S32 => 32, S33 => 33, S34 => 34,
    S35 => 35, S36 => 36, S37 => 37, S38 => 38, S39 => 39,
    S40 => 40, S41 => 41, S42 => 42, S43 => 43, S44 => 44,
    S45 => 45, S46 => 46, S47 => 47, S48 => 48, S49 => 49,
    S50 => 50, S51 => 51, S52 => 52, S53 => 53, S54 => 54,
  }
}
fn main() -> Int {
  if state_to_int(State.S00) != 0 { return 1; }
  if state_to_int(State.S25) != 25 { return 2; }
  if state_to_int(State.S50) != 50 { return 3; }
  if state_to_int(State.S54) != 54 { return 4; }
  return 0;
}
