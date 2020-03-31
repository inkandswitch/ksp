use async_trait::async_trait;
use knowledge_server_base::data::{Link, Tag};
use url::Url;

#[async_trait]
pub trait Resource<'a> {
  type Links: Iterator<Item = Link>;
  type Tags: Iterator<Item = Tag>;

  fn url(&'a self) -> &'a Url;
  async fn links(&'a self) -> Self::Links;
  async fn tags(&'a self) -> Self::Tags;
}
