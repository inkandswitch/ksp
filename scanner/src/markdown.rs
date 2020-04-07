use crate::frontmatter::read_metadata;
use crate::resource::Resource;
use core::ops::Range;
use knowledge_server_base::data::{InputLink, InputResource, LinkKind};
use pulldown_cmark::{Event as Token, LinkType, OffsetIter, Parser, Tag as Span};
use std::io::Result;

pub async fn read(resource: &Resource) -> Result<InputResource> {
    let mut content = String::new();
    resource.read_to_string(&mut content).await?;
    let metadata = read_metadata(&content).await;
    let data = parse(&content, &resource).await;

    let resource = InputResource {
        url: resource.url().to_string(),
        links: Some(data.links),
        tags: metadata.tags,
        title: metadata.title.or(data.title).unwrap_or(format!("")),
        descripton: metadata
            .description
            .or(data.description)
            .unwrap_or(format!("")),
    };

    Ok(resource)
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

#[derive(Debug, Clone)]
pub struct ParseData {
    title: Option<String>,
    description: Option<String>,
    links: Vec<InputLink>,
}

enum LinkContext {
    Heading,
    Paragraph,
    BlockQuote,
    ListItem,
    TableCell,
}

pub async fn parse(source: &str, resource: &Resource) -> ParseData {
    let mut links = vec![];
    let mut title = None;
    let mut description = None;
    let tokens = Parser::new(source).into_offset_iter();
    let mut context: Option<(LinkContext, Range<usize>)> = None;

    for (token, range) in tokens {
        match token {
            // If we encounter header and we have not encountered title yet,
            // this is it. Header acts as link context so we save range in
            // case link is discovered with-in it.
            Token::Start(Span::Heading(level)) => {
                if title.is_none() && level == 1 {
                    // start + 2 to exclude `# ` & `end - 1` to exclude `\n`.
                    title = Some(String::from(&source[range.start + 2..range.end - 1]));
                }
                context = Some((LinkContext::Heading, range));
            }
            Token::End(Span::Heading(_level)) => {
                context = None;
            }

            // Paragraph act as a link context. If it is encountered and we do not
            // have a description this is it.
            Token::Start(Span::Paragraph) => {
                if description.is_none() {
                    description = Some(String::from(&source[range.start..range.end - 1]));
                }

                // If it is BlockQuote we just ignore.
                if let None = context {
                    context = Some((LinkContext::Paragraph, range));
                }
            }
            Token::End(Span::Paragraph) => {
                // If it is not Paragraph we just ignore.
                if let Some((LinkContext::Paragraph, _)) = context {
                    context = None;
                }
            }

            Token::Start(Span::BlockQuote) => {
                context = Some((LinkContext::BlockQuote, range));
            }
            Token::End(Span::BlockQuote) => {
                context = None;
            }

            // List item
            Token::Start(Span::Item) => {
                // Drop `- ` / `* `.
                context = Some((LinkContext::ListItem, range));
            }
            Token::End(Span::Item) => {
                context = None;
            }

            // Links
            Token::Start(Span::Link(_type, _url, _title, _id)) => {}
            Token::End(Span::Link(link_type, url, title, id)) => {
                let context_text = context.as_ref().map(|context| match context {
                    (LinkContext::Heading, range) => source[range.start + 2..range.end].into(),
                    (LinkContext::Paragraph, range) => source[range.start..range.end].into(),
                    (LinkContext::ListItem, range) => source[range.start + 2..range.end].into(),
                    (LinkContext::BlockQuote, range) => {
                        format!("> {:}", &source[range.start..range.end])
                    }
                    (LinkContext::TableCell, range) => source[range.start..range.end].into(),
                });

                println!("{:?}", resource.url().join(&url));
                let link = InputLink {
                    kind: LinkKind::from_type(link_type),
                    target_url: url.into_string(),
                    name: String::from(&source[range.start..range.end]),
                    title: title.into_string(),
                    identifier: {
                        if id.is_empty() {
                            None
                        } else {
                            Some(id.into_string())
                        }
                    },
                    context: context_text,
                };
                links.push(link);
            }

            Token::Start(Span::TableCell) => {
                context = Some((LinkContext::TableCell, range));
            }
            Token::End(Span::TableCell) => {
                context = None;
            }
            _ => {}
        }
    }

    // let title_range = md_title(tokens);
    // let title = title_range.map(|range| String::from(&source[range]));

    ParseData {
        title,
        description,
        links,
    }
}
