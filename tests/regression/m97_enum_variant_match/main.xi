use xiom.io;

enum TrafficLight {
  Red,
  Yellow,
  Green,
}

fn TrafficLight.next(self) -> TrafficLight {
  return match self {
    Red => TrafficLight.Green,
    Green => TrafficLight.Yellow,
    Yellow => TrafficLight.Red,
  };
}

fn TrafficLight.instruction(self) -> Str {
  return match self {
    Red => "Stop",
    Yellow => "Slow",
    Green => "Go",
  };
}

fn main() -> Int {
  let light = TrafficLight.Red;
  let count = 0;
  while count < 3 {
    io.println(light.instruction().to_str());
    light = light.next();
    count = count + 1;
  }
  return 0;
}
