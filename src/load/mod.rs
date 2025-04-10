use futures::StreamExt;
use reqwest::StatusCode;
use serde_yaml::from_str;
use std::{
  collections::HashMap,
  fs::{self, DirEntry},
  io::Result,
  ptr::drop_in_place,
};

use crate::{
  regex::{GITHUB_API, GITHUB_REPO, REGEX_ARM64, REGEX_X86_64},
  structs::{App, ParsedApp, Url, REQWEST},
};

mod cnt;

pub async fn load_all() -> Vec<ParsedApp> {
  let filter = fs::read_dir("./appimage.github.io/apps")
    .expect("Failed to read dir")
    .into_iter()
    .map(load_this);

  let data = tokio_stream::iter(filter)
    .buffer_unordered(20)
    .collect::<Vec<_>>()
    .await;

  let mut failed = 0u32;

  let resp = data
    .into_iter()
    .filter(|x| match x {
      None => {
        failed += 1;
        false
      }
      Some(_) => true,
    })
    .map(|x| x.unwrap())
    .collect();

  println!("❌ Failed: {failed}");

  resp
}

pub async fn load_this(x: Result<DirEntry>) -> Option<ParsedApp> {
  let entry = x.ok()?;

  let name = entry.file_name().into_string().ok()?.replace(".md", "");

  let path = entry.path();

  // Ignore `.` Apps
  if !name.starts_with(".") {
    let meta = fs::read_to_string(&path).ok()?;

    let meta = meta
      .lines()
      .into_iter()
      .filter(|x| return !x.trim().is_empty() && x.trim() != "---")
      .collect::<Vec<_>>()
      .join("\n");

    let meta: App = from_str(&meta).ok()?;

    let icon = get_icon(meta.icons);

    let res = get_screenshots(icon, meta.screenshots);
    let url = format!("./appimage.github.io/data/{}", &name);
    let url = fs::read_to_string(url).ok()?;

    let urls = url.lines().next()?;
    let urls = urls.to_string().leak();

    let ret = ParsedApp {
      name,
      description: meta.description.unwrap_or_default(),
      license: meta.license.unwrap_or_default(),
      screenshots: vec![],
      authors: vec![],
      resources: res,
      url: classify_url(urls).await?,
    };
    return Some(ret);
  }

  None
}

#[async_recursion::async_recursion]
pub async fn classify_url(url: &'static mut str) -> Option<Url> {
  if let Some(caps) = GITHUB_REPO.captures(url) {
    let owner = caps.get(1)?.as_str();
    let repo = caps.get(2)?.as_str();

    let data = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");

    // SAFETY: Since we never ever use it again.... We're safe
    // The `url` should be &'static because if it wasn't GITHUB_REPO type, we'll need it
    unsafe {
      drop_in_place(url);
    }

    return Some(Url::GitHubReleases(data));
  }

  if let Some(caps) = GITHUB_API.captures(url) {
    let owner = caps.get(1)?.as_str();
    let repo = caps.get(2)?.as_str();

    return Some(Url::GitHubReleases(format!(
      "https://api.github.com/repos/{owner}/{repo}/releases/latest"
    )));
  }

  if REGEX_X86_64.is_match(url) {
    return Some(Url::X86_64(url));
  }

  if REGEX_ARM64.is_match(url) {
    return Some(Url::Aarch64(url));
  }

  // Just ditch the rest into X86_64
  if url.ends_with(".AppImage") {
    return Some(Url::X86_64(url));
  }

  // Makes a guesswork
  if url.starts_with("https://") {
    let res = REQWEST.get(&*url).send().await.ok()?;

    if res.status() == StatusCode::OK {
      let filename = cnt::guess_filename(&res)?;

      println!("✅ HTTP Fetched {filename}");

      drop(res);

      return classify_url_adv(&filename, url);
    }
  }

  println!("❌ Couldn't fetch URL: {url}");

  unsafe {
    // SAFETY: Nothing is using it
    drop_in_place(url);
  }

  None
}

fn classify_url_adv(filename: &str, url: &'static mut str) -> Option<Url> {
  if REGEX_X86_64.is_match(filename) {
    return Url::X86_64(url).into();
  }

  if REGEX_ARM64.is_match(filename) {
    return Url::Aarch64(url).into();
  }

  // SAFETY: Completely safe
  unsafe {
    drop_in_place(url);
  }

  return None;
}

pub fn get_icon(icons: Option<Vec<String>>) -> Vec<u8> {
  _get_icon(icons).unwrap_or_else(|| fs::read("./appimage.github.io/img/placeholder.png").unwrap())
}

fn _get_icon(icons: Option<Vec<String>>) -> Option<Vec<u8>> {
  let icons = icons?;
  let loc: &str = icons.iter().find(|x| x.ends_with(".png"))?;

  fs::read(format!("./appimage.github.io/database/{loc}")).ok()
}

pub fn get_screenshots(icon: Vec<u8>, ss: Option<Vec<String>>) -> HashMap<u8, Vec<u8>> {
  let mut hmap = HashMap::new();

  hmap.insert(0, icon);

  let _ = _get_screenshots(&mut hmap, ss);

  hmap
}

fn _get_screenshots(hmap: &mut HashMap<u8, Vec<u8>>, ss: Option<Vec<String>>) -> Option<()> {
  let icons = ss?;
  let loc = icons
    .into_iter()
    .map(|x| fs::read(format!("./appimage.github.io/database/{x}")).ok())
    .filter(|x| x.is_some())
    .map(|x| x.unwrap())
    .collect::<Vec<_>>();

  for (k, val) in loc.into_iter().enumerate() {
    // Only for the 1st 5 ss
    if k < 5 {
      let _ = hmap.insert(k as u8 + 1, val);
    }
  }

  Some(())
}
