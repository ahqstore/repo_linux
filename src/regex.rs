use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};

pub static REGEX: LazyLock<Regex> = LazyLock::new(|| {
  RegexBuilder::new(r"^.*x86_64.*\.AppImage$")
    .case_insensitive(true)
    .build()
    .unwrap()
});