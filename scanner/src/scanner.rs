use async_std::fs;
use dirs;
use ignore;
use std::path::Path;

fn markdown_type() -> Result<ignore::types::Types, ignore::Error> {
  let mut types = ignore::types::TypesBuilder::new();
  types.add("markdown", "*.md")?;
  types.select("markdown");
  types.build()
}

pub async fn scan(path: &Path) {
  let markdown = markdown_type().unwrap();
  let overrides = ignore::overrides::OverrideBuilder::new("")
    .add("!node_modules")
    .unwrap()
    .build()
    .unwrap();
  let walker = ignore::WalkBuilder::new(path)
    .overrides(overrides)
    .standard_filters(true)
    .add_custom_ignore_filename(".ksignore")
    .types(markdown)
    .build();

  walker
    .filter_map(Result::ok)
    .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
    .for_each(|f| println!("{:?}", f.path()))
}

pub async fn activate() {
  println!("Start scanning ~");
  scan(&Path::new("/Users/gozala/Projects")).await;
}
