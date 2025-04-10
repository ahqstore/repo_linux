pub(crate) mod caching;
mod load;
pub(crate) mod parser;
pub(crate) mod regex;
pub(crate) mod structs;

#[tokio::main]
async fn main() {
  caching::purge().await.unwrap();

  let loaded = load::load_all().await;

  parser::parser(loaded).await;
}
