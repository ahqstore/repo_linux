use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};

pub static REGEX_X86_64: LazyLock<Regex> = LazyLock::new(|| {
  RegexBuilder::new(r"^.*x86_64.*\.AppImage$")
    .case_insensitive(true)
    .build()
    .unwrap()
});

pub static REGEX_ARM64: LazyLock<Regex> = LazyLock::new(|| {
  RegexBuilder::new(r"^.*aarch64.*\.AppImage$")
    .case_insensitive(true)
    .build()
    .unwrap()
});