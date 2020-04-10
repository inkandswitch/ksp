use crate::markdown;
use crate::resource::Resource;
use async_std::io;
use ignore::DirEntry;
use knowledge_server_base::data::InputTag;
use knowledge_server_base::schema::{init, FieldError, Mutations};
use std::path::Path;

fn markdown_type() -> Result<ignore::types::Types, ignore::Error> {
    let mut types = ignore::types::TypesBuilder::new();
    types.add("markdown", "*.md")?;
    types.select("markdown");
    types.build()
}

pub fn walk(path: &Path) -> impl Iterator<Item = DirEntry> {
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
}

pub struct ScanReport {
    pub links: usize,
    pub tags: usize,
}

pub enum ScanError {
    IO(io::Error),
    Mutation(FieldError),
}

pub async fn scan<'a>(path: &Path, tags: &Vec<&str>, dry_run: bool) -> io::Result<usize> {
    let entries = walk(path);
    let mut n = 0;
    let service = init()?;

    for entry in entries {
        let path = entry.path();
        let resource = Resource::from_file_path(path)?;
        let mut data = markdown::read(&resource).await?;
        let mut resource_tags = data.tags.unwrap_or(vec![]);

        for tag in tags {
            resource_tags.push(InputTag {
                name: format!("{:}", tag),
                target_fragment: None,
                target_location: None,
            })
        }
        data.tags = Some(resource_tags);

        if dry_run {
            println!("{:#?}", data);
        } else {
            Mutations::ingest(&service, data)
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.message()))?;
            n += 1;
        }
    }

    Ok(n)
}
