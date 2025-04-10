use std::{
  sync::Arc,
  time::{SystemTime, UNIX_EPOCH},
};

use blake3::hash;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use tokio::{
  fs,
  task::{spawn_blocking, JoinSet},
};

use crate::{
  regex::{REGEX_ARM64, REGEX_X86_64},
  structs::REQWEST_AUTH,
};

use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
  pub tag_name: String,
  pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
  pub name: String,
  pub browser_download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parsed {
  pub x86_64: Option<String>,
  pub aarch64: Option<String>,
  pub tag_name: String,
}

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

      if now >= 2 * 60 * 60 + data.time {
        let _ = fs::remove_file(path).await;
      }
    });
  }

  let _ = d.join_all().await;

  Some(())
}

pub async fn fetch(url: Arc<String>) -> Option<Parsed> {
  let mut rng = StdRng::from_os_rng();

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

  let payload = REQWEST_AUTH
    .get(&*url)
    .send()
    .await
    .ok()?;

  if payload.status() != StatusCode::OK {
    let text = payload.text().await.unwrap_or_default();

    println!("⏲️ {text}");

    return None;
  }

  let parsed: Release = payload.json()
    .await
    .ok()?;

  let parsed: Parsed = parse(parsed);

  fs::write(
    format!("./caches/{chk_hash}"),
    to_string(&Cache {
      data: parsed.clone(),
      time: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + rng.random_range(60*60..2*60*60),
    })
    .ok()?,
  )
  .await
  .ok()?;

  Some(parsed)
}

pub fn parse(parsed: Release) -> Parsed {
  let x86_64 = parsed
    .assets
    .iter()
    .find(|x| REGEX_X86_64.is_match(&x.name))
    .and_then(|x| Some(x.browser_download_url.clone()));

  let aarch64 = parsed
    .assets
    .into_iter()
    .find(|x| REGEX_ARM64.is_match(&x.name))
    .and_then(|x| Some(x.browser_download_url));

  Parsed {
    x86_64,
    aarch64,
    tag_name: parsed.tag_name,
  }
}
