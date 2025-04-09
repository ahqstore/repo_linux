use std::{fs::{self, DirEntry}, io::Result};
use serde_yaml::from_str;

use crate::structs::App;

pub fn load_all() -> Vec<()> {
  fs::read_dir("./appimage.github.io/apps").expect("Failed to read dir")
    .into_iter()
    .map(load_this)
    .filter(|x| {
      if !x.is_none() {
        println!("[ERR] Metadata Fetch Failed for 1 app");
      }
      x.is_some()
    })
    .map(|x| x.unwrap())
    .collect()
}

pub fn load_this(x: Result<DirEntry>) -> Option<()> {
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

    println!("{meta:?}");
    return Some(());
  }
  
  None
}