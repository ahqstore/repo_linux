use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};

pub static GITHUB_REPO: LazyLock<Regex> = LazyLock::new(|| {
  RegexBuilder::new(r"^https://github\.com/([a-z0-9\-_\.]*)/([a-z0-9\-_\.]*).*$")
    .case_insensitive(true)
    .build()
    .unwrap()
});

pub static GITHUB_API: LazyLock<Regex> = LazyLock::new(|| {
  RegexBuilder::new(r"^https://api\.github\.com/repos/([a-z0-9\-_\.]*)/([a-z0-9\-_\.]*).*$")
    .case_insensitive(true)
    .build()
    .unwrap()
});

pub static REGEX_X86_64: LazyLock<Regex> = LazyLock::new(|| {
  RegexBuilder::new(r"^.*(x86_64|amd64).*\.AppImage$")
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
