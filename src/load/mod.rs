use std::{collections::HashMap, fs::{self, DirEntry}, io::Result};
use serde_yaml::from_str;

use crate::structs::{App, ParsedApp};

pub fn load_all() -> Vec<ParsedApp> {
  fs::read_dir("./appimage.github.io/apps").expect("Failed to read dir")
    .into_iter()
    .map(load_this)
    .filter(|x| x.is_some())
    .map(|x| x.unwrap())
    .collect()
}

pub fn load_this(x: Result<DirEntry>) -> Option<ParsedApp> {
  let entry = x.ok()?;

  let name = entry.file_name().into_string().ok()?;

  let path = entry.path();

  // Ignore `.` Apps
  if !name.starts_with(".") {
    let meta = fs::read_to_string(&path).ok()?;

    let meta = meta.lines().into_iter()
      .filter(|x| {
        return !x.trim().is_empty() && x.trim() != "---"
      })
      .collect::<Vec<_>>()
      .join("\n");

    let meta: App = from_str(&meta).ok()?;

    let icon = get_icon(meta.icons);
    let res = get_screenshots(icon, meta.screenshots);

    let url = fs::read_to_string(path.join("url")).ok()?;

    let ret = ParsedApp {
      name,
      description: meta.description?,
      license: meta.license?,
      screenshots: vec![],
      authors: vec![],
      resources: res,
      url
    };
    return Some(ret);
  }
  
  None
}

pub fn get_icon(icons: Option<Vec<String>>) -> Vec<u8> {
  _get_icon(icons).unwrap_or_else(|| {
    fs::read("./appimage.github.io/img/placeholder.png").unwrap()
  })
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
  let loc = icons.into_iter()
    .map(|x| {
      fs::read(format!("./appimage.github.io/database/{x}")).ok()
    })
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