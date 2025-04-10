use std::{
  collections::HashMap,
  fs::{self, File},
  io::Write,
  sync::Arc,
};

use ahqstore_types::{
  AHQStoreApplication, AppRepo, DownloadUrl, InstallerFormat, InstallerOptions,
  InstallerOptionsLinux,
};
use blake3::hash;

use crate::{
  caching::{fetch, Parsed},
  structs::{ParsedApp, Url},
};

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
    val.push_str(&format!("l:{}\n", &app_id));

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
      .write(format!("\"{}\":\"l:{}\"", app.appDisplayName, app.appId).as_bytes());
    let _ = self.search.write(
      format!(
        "{{\"name\": {:?}, \"title\": {:?}, \"id\": {:?}}}",
        app.appDisplayName,
        app.appShortcutName,
        format!("l:{}", app.appId)
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

pub async fn parser(apps: Vec<ParsedApp>) {
  println!("⏲️ Please wait...");
  let _ = fs::remove_dir_all("./db");
  let _ = fs::create_dir_all("./db");

  fs::copy("./home.json", "./db/home.json").unwrap();

  let mut map = Map::new();

  for app in apps {
    let _ = parse_push(app, &mut map).await;
  }

  map.finish();
  println!("✅ Done!");
}

async fn parse_push(app: ParsedApp, map: &mut Map) -> Option<()> {
  let mut version = hash(format!("l:{}:{:#?}", app.name, &app.url).as_bytes()).to_string();

  let mut aarch64 = None;
  let mut x86_64 = None;

  let mut downloads = HashMap::new();

  match app.url {
    Url::Aarch64(x) => {
      aarch64 = Some(InstallerOptionsLinux { assetId: 1 });
      downloads.insert(
        1,
        DownloadUrl {
          installerType: InstallerFormat::LinuxAppImage,
          url: x.to_string(),
          asset: format!("<-- NO NEED -->"),
        },
      );
    }
    Url::X86_64(x) => {
      x86_64 = Some(InstallerOptionsLinux { assetId: 0 });

      downloads.insert(
        0,
        DownloadUrl {
          installerType: InstallerFormat::LinuxAppImage,
          url: x.to_string(),
          asset: format!("<-- NO NEED -->"),
        },
      );
    }
    Url::GitHubReleases(url) => {
      println!("{url}");
      let url = Arc::new(url);

      let Parsed {
        x86_64: amd64,
        aarch64: arm64,
        tag_name,
      } = fetch(url).await?;

      version = hash(format!("l:{}:{tag_name}", version).as_bytes()).to_string();

      if let Some(amd64) = amd64 {
        x86_64 = Some(InstallerOptionsLinux { assetId: 0 });

        downloads.insert(
          0,
          DownloadUrl {
            installerType: InstallerFormat::LinuxAppImage,
            url: amd64,
            asset: format!("<-- NO NEED -->"),
          },
        );
      }

      if let Some(arm64) = arm64 {
        x86_64 = Some(InstallerOptionsLinux { assetId: 0 });

        downloads.insert(
          0,
          DownloadUrl {
            installerType: InstallerFormat::LinuxAppImage,
            url: arm64,
            asset: format!("<-- NO NEED -->"),
          },
        );
      }
    }
  }

  let app = AHQStoreApplication {
    version,
    appDisplayName: app.name.clone(),
    appShortcutName: app.name.clone(),
    appId: app.name,
    authorId: "6adfb183a4a2c94a2f92dab5ade762a47889a5a1".into(),
    description: app.description,
    displayImages: app
      .resources
      .iter()
      .map(|(k, _)| *k)
      .filter(|x| *x != 0)
      .collect::<Vec<_>>(),
    resources: Some(app.resources),
    releaseTagName: format!("ahqstore"),
    license_or_tos: Some(app.license),
    site: None,
    repo: AppRepo {
      author: format!("AppImage"),
      repo: format!("appimage.github.io"),
    },
    source: Some("appimages".into()),
    verified: false,
    install: InstallerOptions {
      android: None,
      winarm: None,
      win32: None,
      linuxArm7: None,
      linux: x86_64,
      linuxArm64: aarch64,
    },
    downloadUrls: downloads,
  };

  map.add(app);

  Some(())
}
