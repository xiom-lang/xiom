// M19-R01: io.read_file() regression test
// Verifies: read_file returns correct content, not empty string
use stdlib.xiom.io;

fn main() -> Int {
  // Write test file with known content
  var write_result = io.write_file("__m19_read_test.txt", "M19-PASS-42");
  match write_result {
    Err(e) => { io.println("FAIL: write error: " + e.message); return 1; }
    _ => {}
  }
  
  // Read it back
  var read_result = io.read_file("__m19_read_test.txt");
  match read_result {
    Ok(content) => {
      // Clean up
      var _ = io.remove_file("__m19_read_test.txt");
      if content == "M19-PASS-42" { return 0; }
      io.println("FAIL: got=[" + content + "] expected=[M19-PASS-42]");
      return 1;
    }
    Err(e) => {
      io.println("FAIL: read error: " + e.message);
      return 1;
    }
  }
}
