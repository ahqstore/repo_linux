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
          $x: Option<$t>
        ),*
      }
    )*
  };
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tags {
  #[serde(rename = "thirdparty")]
  ThirdParty,
  #[serde(rename = "official")]
  Official
}

define! {
  App {
    published: bool,
    layout: String,
    created: String,
    updated: String,
    description: String,
    generic: String,
    license: String,
    authors: Vec<Author>,
    screenshots: Vec<String>,
    tags: Vec<Tags>
  },
  Author {
    name: String,
    url: String
  }
}