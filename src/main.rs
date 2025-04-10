pub(crate) mod regex;
pub(crate) mod structs;
pub(crate) mod parser;
mod load;

#[tokio::main]
async fn main() {
  load::load_all();
  // tokio_stream::iter();
  
  parser::parser().await;
}
