use crate::markdown::MarkdownResource;
use crate::resource::Resource;
use async_std::fs::File;
use async_std::io;
use async_std::prelude::*;
use dirs;
use ignore;
use std::path::Path;

fn markdown_type() -> Result<ignore::types::Types, ignore::Error> {
  let mut types = ignore::types::TypesBuilder::new();
  types.add("markdown", "*.md")?;
  types.select("markdown");
  types.build()
}

pub async fn scan(path: &Path) -> io::Result<()> {
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

  let entries = walker
    .filter_map(Result::ok)
    .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false));

  for entry in entries {
    let path = entry.path();
    let resource = MarkdownResource::try_from_file_path(&path).await?;

    println!("---------------- Links -------------------");
    for link in resource.links().await {
      println!("{:?}", link);
    }
    println!("---------------- Links -------------------");

    for tag in resource.tags().await {
      println!("{:?}", tag);
    }

    //     let mut file = File::open(path).await?;
    //     let mut contents = String::new();
    //     file.read_to_string(&mut contents).await?;
    //     let contents2 = "# Test
    // [ref-link][link-id]
    // [shortcut]
    // [collapse][]
    // [inline](https://my.link/inline)
    // [`code` **test** ~~bla~~][link-id]

    // [link-id]:https://my.link/foo \"my link\"
    // [shortcut]:dat:///shortcut
    // [collapse]:https://my.link/collapse
    // [obsolete]:https://my.link/obsolete-link
    // ";
    //     find_links(&path).await?;
    // file.read_to_end(&mut contents).await?;
    // println!("{:?}", file);
  }

  Ok(())
}

pub async fn activate() -> io::Result<()> {
  println!("Start scanning ~");
  scan(&Path::new("/Users/gozala/Sites/Notes")).await?;
  Ok(())
}
