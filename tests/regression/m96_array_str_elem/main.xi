use xiom.io;
use xiom.iter;

fn main() {
  let friends = ["Maya", "Leo", "Priya", "Kai"];

  io.println("Greeting my friends:");

  for __i in range(0, friends.len()) {
    let friend = friends[__i];
    let message = "Hi, " + friend + "!";
    io.println(message);
  }

  io.println("Everyone has been greeted!");
}
