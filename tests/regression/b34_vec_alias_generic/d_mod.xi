module d_mod

pub type Handle = Int;

pub fn take_handles(xs: &Vec[Handle]) -> Int {
  return xs.len();
}

pub fn make_handle() -> Handle {
  return 42;
}
