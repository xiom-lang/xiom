fn main() {
  match io.read_line() {
    Ok(line) => { io.print("got: "); io.println(line); },
    Err(_) => { io.println("error"); },
  }
}
