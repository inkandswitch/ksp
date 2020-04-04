use crate::resource::Resource;
use frontmatter;
use knowledge_server_base::data::{InputLink, InputResource, LinkKind};
use pulldown_cmark::{Event as Token, LinkType, Parser, Tag as Span};
use std::io::Result;
use yaml_rust::yaml::{Hash, Yaml};

pub async fn read(resource: &Resource) -> Result<InputResource> {
  let mut content = String::new();
  resource.read_to_string(&mut content).await?;
  let resource = InputResource {
    url: resource.url().to_string(),
    links: read_links(&content).await,
    tags: read_tags(&content).await,
  };

  Ok(resource)
}

pub async fn read_links(source: &str) -> Option<Vec<InputLink>> {
  Some(Links::parse(source).collect())
}

pub async fn read_tags(source: &str) -> Option<Vec<String>> {
  let metadata = frontmatter::parse(&source)
    .unwrap_or(None)
    .and_then(|data| data.into_hash());

  metadata.as_ref().and_then(get_tags).and_then(decode_tags)
}

fn get_tags(map: &Hash) -> Option<&Yaml> {
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

/// Represents iterator of links found in the markdown file.
pub struct Links<'a> {
  /// Parsed markdown document tokens.
  tokens: Parser<'a>,
}
impl<'a> Links<'a> {
  fn parse(content: &'a str) -> Self {
    Links {
      tokens: Parser::new(content),
    }
  }
  // Consumes tokens until `Link` is fonud.
  fn read_link(&mut self) -> Option<InputLink> {
    loop {
      if let Some(token) = self.tokens.next() {
        match token {
          Token::Start(Span::Link(link_type, url, title, id)) => {
            let text = self.read_text();
            let link = InputLink {
              kind: LinkKind::from_type(link_type),
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
  type Item = InputLink;
  fn next(&mut self) -> Option<Self::Item> {
    self.read_link()
  }
}
