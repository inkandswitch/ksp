mod data_store;

use async_std::task;
use data_store::{DataStore, Link, Tag};
use juniper::{EmptyMutation, FieldResult, RootNode};
use tide::{Request, Response, Server};

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

#[derive(juniper::GraphQLEnum, Clone, Copy)]
enum LinkKind {
  Inline = 0,
  Reference = 1,
}

/// Represents an inline link in markdown file.
#[juniper::graphql_object(Context = State)]
impl Link {
  fn kind(&self) -> LinkKind {
    match self.kind {
      data_store::LinkKind::Inline => LinkKind::Inline,
      data_store::LinkKind::Reference => LinkKind::Reference,
    }
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

struct Resource {
  url: String,
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

  /// Tag associated to this document
  async fn tags(&self, state: &State) -> FieldResult<Vec<Tag>> {
    state.store.tags_by_target(&self.url)
  }

  // Similar resources
  async fn similar(&self, _state: &State) -> Vec<SimilarResource> {
    vec![]
  }
}

struct SimilarResource {
  // URL of the similar resource
  target: String,
}
#[juniper::graphql_object(Context = State)]
impl SimilarResource {
  async fn target(&self) -> Resource {
    Resource {
      url: self.target.clone(),
    }
  }
}

struct Query;

#[juniper::graphql_object(Context = State)]
impl Query {
  async fn lookup(_state: &State, url: String) -> Resource {
    Resource { url: url }
  }
}

type Schema = RootNode<'static, Query, EmptyMutation<State>>;
fn schema() -> Schema {
  Schema::new(Query, EmptyMutation::<State>::new())
}

async fn handle_graphiql(_: Request<State>) -> Response {
  Response::new(200)
    .body_string(juniper::http::graphiql::graphiql_source("/graphql"))
    .set_header("content-type", "text/html;charset=utf-8")
}

async fn handle_graphql(mut cx: Request<State>) -> Response {
  task::block_on(async {
    let query: juniper::http::GraphQLRequest = cx
      .body_json()
      .await
      .expect("be able to deserialize the graphql request");

    let schema = schema(); // probably worth making the schema a singleton using lazy_static library
    let response = query.execute_async(&schema, &cx.state()).await;
    let status = if response.is_ok() { 200 } else { 400 };

    Response::new(status)
      .body_json(&response)
      .expect("be able to serialize the graphql response")
  })
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
  let store = DataStore::open()?;

  let state = State { store: store };
  let mut service = Server::with_state(state);
  service.at("/").get(tide::redirect("/graphiql"));
  service.at("/graphql").post(handle_graphql);
  service.at("/graphiql").get(handle_graphiql);
  service.listen("0.0.0.0:8080").await?;
  Ok(())
}
