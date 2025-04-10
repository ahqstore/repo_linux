use std::{
  sync::Arc,
  time::{SystemTime, UNIX_EPOCH},
};

use blake3::hash;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use tokio::{
  fs,
  task::{spawn_blocking, JoinSet},
};

use crate::structs::REQWEST_AUTH;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parsed {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cache {
  pub time: u64,
  pub data: Parsed,
}

pub async fn purge() -> Option<()> {
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

  let mut dir = fs::read_dir("./caches").await.ok()?;

  let mut d = JoinSet::new();

  while let Some(entry) = dir.next_entry().await.ok()? {
    d.spawn(async move {
      let path = entry.path();

      let data: Cache = from_str(&fs::read_to_string(&path).await.unwrap()).unwrap();

      if now - data.time >= 2 * 60 * 60 {
        let _ = fs::remove_file(path).await;
      }
    });
  }

  let _ = d.join_all().await;

  Some(())
}

pub async fn fetch(url: Arc<str>) -> Option<Parsed> {
  let url2 = url.clone();
  let chk_hash = spawn_blocking(move || hash(url2.as_bytes()))
    .await
    .ok()?
    .to_string();

  if let Some(x) = fs::read_to_string(format!("./caches/{}", &chk_hash))
    .await
    .ok()
    .and_then(|x| from_str::<Cache>(&x).ok())
  {
    return Some(x.data);
  }

  let parsed: Parsed = REQWEST_AUTH
    .get(&*url)
    .send()
    .await
    .ok()?
    .json()
    .await
    .ok()?;

  fs::write(
    format!("./caches/{chk_hash}"),
    to_string(&Cache {
      data: parsed.clone(),
      time: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs(),
    })
    .ok()?,
  )
  .await
  .ok()?;

  Some(parsed)
}
