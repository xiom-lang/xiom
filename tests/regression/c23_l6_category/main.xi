// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// C23 lock, l6-15 shape (playground tools/compiler-repros/c23): nested user
// modules, a struct method, and Str-field compares over a Vec. On hosts with
// LLVM 18 the driver's old double pipeline (opt -O1+ then clang -O2) counted
// 3 personal notes instead of 2. Returns 12 when the count is wrong.
// NOTE: no top-level `module` header -- a header breaks the nested-module
// cross-type field compare on current checkers (filed in COMPILER_BUGS).

use xiom.io;

module models {
  pub type Note = {
    content: Str;
    category: Str;
  }

  pub fn new_note(content: Str, category: Str) -> Note {
    return Note{ content: content, category: category };
  }
}

module storage {
  pub type Notebook = {
    notes: Vec[models.Note];
  }

  pub fn new_notebook() -> Notebook {
    return Notebook{ notes: Vec[models.Note].new() };
  }

  pub fn Notebook.add_note(&mut self, content: Str, category: Str) {
    self.notes.push(models.new_note(content, category));
  }

  pub fn Notebook.count(self) -> Int {
    return self.notes.len();
  }

  pub fn count_by_category(notebook: &Notebook, cat: Str) -> Int {
    var count = 0;
    var i = 0;
    while i < notebook.notes.len() {
      let note = notebook.notes[i];
      if note.category == cat {
        count = count + 1;
      }
      i = i + 1;
    }
    return count;
  }
}

fn main() -> Int {
  var nb = storage.new_notebook();
  nb.add_note("Buy milk", "personal");
  nb.add_note("Fix bug", "work");
  nb.add_note("Call mom", "personal");

  let personal = storage.count_by_category(&nb, "personal");
  io.println(personal.to_str());
  if personal != 2 { return 12; }
  return 0;
}
