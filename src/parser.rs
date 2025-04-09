use std::{
  collections::HashMap, fs::{self, File}, io::Write
};

use ahqstore_types::AHQStoreApplication;

struct Map {
  entries: usize,
  files: usize,
  c_file: File,
  search: File,
}

impl Map {
  fn new() -> Self {
    let _ = fs::create_dir_all("./db/map");
    let _ = fs::create_dir_all("./db/search");
    let _ = fs::create_dir_all("./db/apps");
    let _ = fs::create_dir_all("./db/dev");
    let _ = fs::create_dir_all("./db/res");

    let mut file = File::create("./db/map/1.json").unwrap();
    let _ = file.write(b"{");

    let mut search = File::create("./db/search/1.json").unwrap();
    let _ = search.write(b"[");

    Self {
      entries: 0,
      files: 1,
      c_file: file,
      search,
    }
  }

  fn close_file(&mut self) {
    let _ = self.search.write_all(b"]");
    let _ = self.search.flush();
    let _ = self.c_file.write_all(b"}");
    let _ = self.c_file.flush();
  }

  fn new_file(&mut self) {
    self.files += 1;
    self.entries = 0;
    self.close_file();

    let mut map = File::create("./db/map/1.json").unwrap();
    let _ = map.write(b"{");

    let mut search = File::create("./db/map/1.json").unwrap();
    let _ = search.write(b"[");

    self.c_file = map;
    self.search = search;
  }

  fn add_author(&mut self, author: &str, app_id: &str) {
    let file = format!("./db/dev/{}", author);
    let mut val = fs::read_to_string(&file).unwrap_or("".to_string());
    val.push_str(&format!("f:{}\n", &app_id));

    let _ = fs::write(&file, val);
  }

  fn add(&mut self, mut app: AHQStoreApplication) {
    if self.entries >= 100_000 {
      self.new_file();
    }
    println!("{}", self.entries);
    if self.entries > 0 {
      let _ = self.c_file.write(b",");
      let _ = self.search.write(b",");
    }

    self.add_author(&app.authorId, &app.appId);
    self.entries += 1;

    let _ = self
      .c_file
      .write(format!("\"{}\":\"f:{}\"", app.appDisplayName, app.appId).as_bytes());
    let _ = self.search.write(
      format!(
        "{{\"name\": {:?}, \"title\": {:?}, \"id\": {:?}}}",
        app.appDisplayName, app.appShortcutName, format!("f:{}", app.appId)
      )
      .as_bytes(),
    );

    let (app_str, res) = app.export();

    let path = format!("./db/apps/{}.json", &app.appId);

    let _ = fs::create_dir_all(format!("./db/res/{}", &app.appId));

    for (id, bytes) in res {
      let _ = fs::write(format!("./db/res/{}/{}", &app.appId, id), bytes);
    }

    app.appId = format!("f:{}", app.appId);
    app.authorId = format!("f:{}", app.authorId);

    println!("✅ Adding {}", &app.appId);

    let _ = fs::write(path, app_str);
  }

  fn finish(mut self) {
    self.close_file();

    let _ = fs::write("./db/total", self.files.to_string());
  }
}

pub async fn parser() {
  println!("⏲️ Please wait...");
  let _ = fs::remove_dir_all("./db");
  let _ = fs::create_dir_all("./db");

  fs::copy("./home.json", "./db/home.json").unwrap();

  let mut map = Map::new();

  // for (id, meta) in meta.packages {
  //   // Use map.add
  // }

  map.finish();
  println!("✅ Done!");
}
