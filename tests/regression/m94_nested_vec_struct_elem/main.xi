use xiom.io;
// R49 lock: L8-15/L8-18 -- Vec[Vec[Cell]] get/unwrap/set: struct/enum
// elements are boxed on get and unboxed on unwrap; inline set must memcpy
// the element bytes (was clang i64-vs-ptr, then AV).
enum Cell {
  Empty,
  X,
  O,
}

type TicTacBoard = {
  grid: Vec[Vec[Cell]];
}

fn new_board() -> TicTacBoard {
  var grid: Vec[Vec[Cell]] = Vec[Vec[Cell]].new();
  var r = 0;
  while r < 3 {
    var row: Vec[Cell] = Vec[Cell].new();
    row.push(Cell.Empty); row.push(Cell.Empty); row.push(Cell.Empty);
    grid.push(row);
    r = r + 1;
  };
  TicTacBoard{ grid: grid }
}

fn place_mark(board: &mut TicTacBoard, row: Int, col: Int, mark: Cell) -> Bool {
  let current = board.grid.get(row).unwrap().get(col).unwrap();
  if current != Cell.Empty { return false; };
  board.grid.get(row).unwrap().set(col, mark);
  true
}

fn check_winner(board: &TicTacBoard) -> Cell {
  var r = 0;
  while r < 3 {
    let a = board.grid.get(r).unwrap().get(0).unwrap();
    let b = board.grid.get(r).unwrap().get(1).unwrap();
    let c = board.grid.get(r).unwrap().get(2).unwrap();
    if a != Cell.Empty && a == b && b == c { return a; };
    r = r + 1;
  };
  var c = 0;
  while c < 3 {
    let a = board.grid.get(0).unwrap().get(c).unwrap();
    let b = board.grid.get(1).unwrap().get(c).unwrap();
    let cc = board.grid.get(2).unwrap().get(c).unwrap();
    if a != Cell.Empty && a == b && b == cc { return a; };
    c = c + 1;
  };
  let mid = board.grid.get(1).unwrap().get(1).unwrap();
  if mid != Cell.Empty {
    let tl = board.grid.get(0).unwrap().get(0).unwrap();
    let br = board.grid.get(2).unwrap().get(2).unwrap();
    if mid == tl && tl == br { return mid; };
    let tr = board.grid.get(0).unwrap().get(2).unwrap();
    let bl = board.grid.get(2).unwrap().get(0).unwrap();
    if mid == tr && tr == bl { return mid; };
  };
  Cell.Empty
}

fn show_board(board: &TicTacBoard) {
  var r = 0;
  while r < 3 {
    let a = board.grid.get(r).unwrap().get(0).unwrap();
    let b = board.grid.get(r).unwrap().get(1).unwrap();
    let c = board.grid.get(r).unwrap().get(2).unwrap();
    io.println(cell_display(a) + cell_display(b) + cell_display(c));
    r = r + 1;
  };
}

fn cell_display(c: Cell) -> Str {
  match c {
    Cell.Empty => ".",
    Cell.X => "X",
    Cell.O => "O",
  }
}

fn main() -> Int {
  var board = new_board();
  place_mark(&mut board, 0, 0, Cell.X);
  place_mark(&mut board, 1, 1, Cell.O);
  place_mark(&mut board, 0, 1, Cell.X);
  place_mark(&mut board, 2, 2, Cell.O);
  place_mark(&mut board, 0, 2, Cell.X);
  show_board(&board);
  let winner = check_winner(&board);
  if winner == Cell.X { io.println("X wins!"); };
  if winner == Cell.O { io.println("O wins!"); };
  if winner == Cell.Empty { io.println("No winner yet"); };
  return 0;
}
