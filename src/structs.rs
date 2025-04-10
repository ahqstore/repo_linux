use std::{collections::HashMap, sync::LazyLock, time::Duration};

use reqwest::{
  header::{HeaderMap, HeaderValue},
  redirect::Policy,
  Client, ClientBuilder,
};
use serde::{Deserialize, Serialize};

#[cfg(debug_assertions)]
static TIMEOUT: u64 = 1;

#[cfg(not(debug_assertions))]
static TIMEOUT: u64 = 60;

pub static REQWEST: LazyLock<Client> = LazyLock::new(|| {
  ClientBuilder::new()
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.212 Safari/537.36")
    .timeout(Duration::from_secs(TIMEOUT))
    .redirect(Policy::limited(20))
    .build()
    .unwrap()
});

pub static REQWEST_AUTH: LazyLock<Client> = LazyLock::new(|| {
  ClientBuilder::new()
    .default_headers({
      let mut h = HeaderMap::new();

      if let Some(x) = option_env!("GH_TOKEN") {
        h.insert(
          "Authorization",
          HeaderValue::from_str(&format!("Bearer {x}")).unwrap(),
        );
      }

      h
    })
    .timeout(Duration::from_secs(TIMEOUT))
    .user_agent("AHQ Store Linux")
    .build()
    .unwrap()
});

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

#[derive(Debug, Serialize)]
pub struct ParsedApp {
  pub name: String,
  pub description: String,
  pub license: String,
  pub authors: Vec<Author>,
  pub screenshots: Vec<String>,
  pub resources: HashMap<u8, Vec<u8>>,
  pub url: Url,
}

#[derive(Debug, Serialize)]
pub enum Url {
  GitHubReleases(String),
  X86_64(&'static str),
  Aarch64(&'static str),
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
