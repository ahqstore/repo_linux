use std::collections::HashMap;

use serde::{Deserialize, Serialize};

macro_rules! define {
  (
    $(
      $name:ident {
        $($x:ident: $t:ty),*
      }
    ),*
  ) => {
    $(
      #[derive(Debug, Serialize, Deserialize)]
      pub struct $name {
        $(
          pub $x: Option<$t>
        ),*
      }
    )*
  };
}

pub struct ParsedApp {
  pub name: String,
  pub description: String,
  pub license: String,
  pub authors: Vec<Author>,
  pub screenshots: Vec<String>,
  pub resources: HashMap<u8, Vec<u8>>,
  pub url: String
}

define! {
  App {
    published: bool,
    layout: String,
    created: String,
    updated: String,
    description: String,
    license: String,
    authors: Vec<Author>,
    icons: Vec<String>,
    screenshots: Vec<String>
  },
  Author {
    name: String,
    url: String
  }
}