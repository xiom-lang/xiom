// Cross-file sibling module for the m176 qualified-literal lock.
module qlib

pub type LabelParts = {
  key: Str;
  value: Str;
}

pub fn mk(key: Str, value: Str) -> LabelParts {
  return LabelParts{ key: key; value: value; };
}
