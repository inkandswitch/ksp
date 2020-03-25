use crate::data::{
  InputResource, Link, LinkKind, Mutations, Query, Resource, SimilarResource, Tag,
};
use crate::store::DataStore;
use juniper::{FieldResult, RootNode};
use std::io;

pub struct State {
  store: DataStore,
}
impl juniper::Context for State {}

/// Resource tag. For files resources on MacOS/iOS that roughly translates to file / directory tag.
// For web resources that roughly translates to bookmark tags.
#[juniper::graphql_object(Context = State)]
impl Tag {
  fn tag(&self) -> &str {
    &self.tag
  }
  fn target_url(&self) -> &str {
    &self.target_url
  }
}

/// Represents an inline link in markdown file.
#[juniper::graphql_object(Context = State)]
impl Link {
  fn kind(&self) -> LinkKind {
    self.kind
  }
  /// Name that link was encountered by
  fn name(&self) -> &str {
    &self.name
  }
  /// Titile that link was encountered by
  fn title(&self) -> &str {
    &self.title
  }

  /// Label it is referred by
  /// e.g. In [Local-first software][local-first] it is "local-first"
  fn identifier(&self) -> Option<String> {
    match &self.identifier {
      Some(name) => Some(String::from(name)),
      None => None,
    }
  }

  // Target resource of the link
  async fn target(&self) -> Resource {
    Resource {
      url: self.target_url.clone(),
    }
  }
  // Referrer resource
  async fn referrer(&self) -> Resource {
    Resource {
      url: self.referrer_url.clone(),
    }
  }
}

#[juniper::graphql_object(Context = State)]
impl Resource {
  /// URL of the resource
  fn url(&self) -> &str {
    &self.url
  }

  /// Resources this document links to
  async fn links(&self, state: &State) -> FieldResult<Vec<Link>> {
    state.store.links_by_referrer(&self.url)
  }

  // Resources that link to this document
  async fn backLinks(&self, state: &State) -> FieldResult<Vec<Link>> {
    state.store.links_by_target(&self.url)
  }

  // Tag associated to this document
  async fn tags(&self, state: &State) -> FieldResult<Vec<Tag>> {
    state.store.tags_by_target(&self.url)
  }

  // Similar resources
  async fn similar(&self, _state: &State) -> Vec<SimilarResource> {
    vec![]
  }
}

#[juniper::graphql_object(Context = State)]
impl SimilarResource {
  async fn target(&self) -> Resource {
    Resource {
      url: self.target.clone(),
    }
  }
}

#[juniper::graphql_object(Context = State)]
impl Query {
  async fn lookup(_state: &State, url: String) -> Resource {
    Resource { url: url }
  }
}

#[juniper::graphql_object(Context = State)]
impl Mutations {
  /// Injests resource into knowledge base.
  async fn ingest(state: &State, resource: InputResource) -> FieldResult<Resource> {
    if let Some(tags) = resource.tags {
      state.store.add_tags(&resource.url, tags)?;
    }
    if let Some(links) = resource.links {
      state.store.add_links(&resource.url, links)?;
    }
    Ok(Resource { url: resource.url })
  }
}

type Schema = RootNode<'static, Query, Mutations>;
pub fn schema() -> Schema {
  Schema::new(Query, Mutations)
}

pub fn init() -> io::Result<State> {
  let store = DataStore::open()?;

  Ok(State { store: store })
}
