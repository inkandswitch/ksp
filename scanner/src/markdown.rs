use crate::resource::Resource;
use async_std::fs::File;
use async_std::prelude::*;
use async_trait::async_trait;
use frontmatter;
use knowledge_server_base::data::{Link, LinkKind, Tag};
use pulldown_cmark::{Event as Token, LinkType, Parser, Tag as Span};
use std::convert::TryFrom;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use url::Url;
use yaml_rust::yaml::{Array, Hash, Yaml};

pub(crate) struct MarkdownResource {
  url: Url,
  content: String,
}

impl<'a> MarkdownResource {
  pub async fn try_from_file_path(path: &Path) -> Result<MarkdownResource> {
    let url = Url::from_file_path(path)
      .map_err(|_| Error::new(ErrorKind::Other, "Unable to create file URL for path"))?;
    let mut file = File::open(path).await?;
    let mut content = String::new();
    file.read_to_string(&mut content).await?;
    let resource = MarkdownResource { content, url };

    Ok(resource)
  }
}

#[async_trait]
impl<'a> Resource<'a> for MarkdownResource {
  type Links = Links<'a>;
  type Tags = Tags<'a>;
  fn url(&'a self) -> &'a Url {
    &self.url
  }
  async fn links(&'a self) -> Self::Links {
    let parser = Parser::new(&self.content);
    let links = Links {
      tokens: parser,
      referrer_url: &self.url,
    };
    links
  }
  async fn tags(&'a self) -> Self::Tags {
    let metadata = frontmatter::parse(&self.content)
      .unwrap_or(None)
      .and_then(|data| data.into_hash());

    let tagNames = metadata
      .as_ref()
      .and_then(read_tags)
      .and_then(decode_tags)
      .unwrap_or(vec![]);

    let tags = tagNames.into_iter();

    // println!("{:?}", boxedTags);

    Tags {
      // metadata: &metadata,
      tags: tags,
      target_url: &self.url,
    }
  }
}

fn read_tags(map: &Hash) -> Option<&Yaml> {
  map
    .get(&Yaml::String(format!("tags")))
    .or_else(|| map.get(&Yaml::String(format!("Tags"))))
}

fn decode_tags(value: &Yaml) -> Option<Vec<String>> {
  decode_tags_array(value).or_else(|| decode_tags_string(value))
}

fn decode_tags_array(value: &Yaml) -> Option<Vec<String>> {
  value.as_vec().as_ref().map(|vec| {
    vec
      .into_iter()
      .filter_map(decode_tag)
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect()
  })
}

fn decode_tags_string(value: &Yaml) -> Option<Vec<String>> {
  value.as_str().as_ref().map(|s| {
    s.rsplit(",")
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect()
  })
}

fn decode_tag(value: &Yaml) -> Option<&str> {
  if let Yaml::String(tag) = value {
    Some(tag)
  } else {
    None
  }
}

trait LinkKindExt {
  /// Translates link type from [pulldown-cmark][] representation to
  /// representation used by knowledge-server.
  /// [pulldown-cmark]:https://crates.io/crates/pulldown-cmark
  fn from_type(link_type: LinkType) -> Self;
}

impl LinkKindExt for LinkKind {
  fn from_type(link_type: LinkType) -> Self {
    match link_type {
      LinkType::Inline => LinkKind::Inline,
      LinkType::Reference => LinkKind::Reference,
      LinkType::ReferenceUnknown => LinkKind::Reference,
      LinkType::Collapsed => LinkKind::Reference,
      LinkType::CollapsedUnknown => LinkKind::Reference,
      LinkType::Shortcut => LinkKind::Reference,
      LinkType::ShortcutUnknown => LinkKind::Reference,
      LinkType::Autolink => LinkKind::Inline,
      LinkType::Email => LinkKind::Inline,
    }
  }
}

pub struct Tags<'a> {
  target_url: &'a Url,
  tags: std::vec::IntoIter<String>,
}
impl<'a> Iterator for Tags<'a> {
  type Item = Tag;
  fn next(&mut self) -> Option<Self::Item> {
    if let Some(tag) = self.tags.next() {
      Some(Tag {
        target_url: self.target_url.to_string(),
        tag,
      })
    } else {
      None
    }
  }
}

/// Represents iterator of links found in the markdown file.
pub struct Links<'a> {
  /// URL of the markdown document where links are found.
  referrer_url: &'a Url,
  /// Parsed markdown document tokens.
  tokens: Parser<'a>,
}
impl<'a> Links<'a> {
  // Consumes tokens until `Link` is fonud.
  fn read_link(&mut self) -> Option<Link> {
    loop {
      if let Some(token) = self.tokens.next() {
        match token {
          Token::Start(Span::Link(link_type, url, title, id)) => {
            let text = self.read_text();
            let link = Link {
              kind: LinkKind::from_type(link_type),
              referrer_url: self.referrer_url.as_str().to_string(),
              target_url: url.into_string(),
              name: text,
              title: title.into_string(),
              identifier: {
                if id.is_empty() {
                  None
                } else {
                  Some(id.into_string())
                }
              },
            };
            return Some(link);
          }
          _ => {
            continue;
          }
        }
      } else {
        return None;
      }
    }
  }
  // Seralizes tokens until end of the `Link` is found.
  fn read_text(&mut self) -> String {
    let mut string = String::new();
    loop {
      if let Some(token) = self.tokens.next() {
        match token {
          Token::End(Span::Link(_type, _url, _title, _id)) => {
            break;
          }
          Token::Start(Span::Strikethrough) => string.push_str("~~"),
          Token::End(Span::Strikethrough) => string.push_str("~~"),
          Token::Start(Span::Strong) => string.push_str("**"),
          Token::End(Span::Strong) => string.push_str("**"),
          Token::Start(Span::Emphasis) => string.push_str("_"),
          Token::End(Span::Emphasis) => string.push_str("_"),
          Token::Text(text) => string.push_str(text.as_ref()),
          Token::Code(text) => {
            string.push_str("`");
            string.push_str(text.as_ref());
            string.push_str("`");
          }
          _ => {
            continue;
          }
        }
      }
    }
    string
  }
}
impl<'a> Iterator for Links<'a> {
  type Item = Link;
  fn next(&mut self) -> Option<Self::Item> {
    self.read_link()
  }
}
