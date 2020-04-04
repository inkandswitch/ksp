use crate::schema::{schema, State};
use async_std::task;
use async_trait::async_trait;
use futures::future::BoxFuture;
use std::sync::RwLock;
use tide::{Middleware, Next, Request, Response, Server};

async fn handle_graphiql(_: Request<State>) -> Response {
  Response::new(200)
    .body_string(juniper::http::graphiql::graphiql_source("/graphql"))
    .set_header("content-type", "text/html;charset=utf-8")
}

async fn handle_graphql(mut cx: Request<State>) -> Response {
  // Need to do this because future returned by juniper is not sendable,
  // which I think is because it has not benig updated to use never versions
  // of futures.
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

async fn handle_root_head(_request: Request<State>) -> Response {
  Response::new(200)
}

struct Header {
  key: &'static str,
  value: &'static str,
}

struct Headers {
  headers: RwLock<Vec<Header>>,
}

impl Headers {
  fn new() -> Self {
    let headers = vec![];
    Headers {
      headers: RwLock::new(headers),
    }
  }
  fn set(mut self, key: &'static str, value: &'static str) -> Self {
    if let Ok(headers) = self.headers.get_mut() {
      headers.push(Header { key, value })
    }
    self
  }
}

#[async_trait]
impl<State: Send + Sync + 'static> Middleware<State> for Headers {
  fn handle<'a>(
    &'a self,
    request: Request<State>,
    next: Next<'a, State>,
  ) -> BoxFuture<'a, Response> {
    Box::pin(async move {
      let mut response = next.run(request).await;
      if let Ok(headers) = self.headers.read() {
        for header in headers.iter() {
          response = response.set_header(header.key, header.value);
        }
        response
      } else {
        response
      }
    })
  }
}

pub async fn activate(address: &str) -> std::io::Result<()> {
  let state = crate::schema::init()?;
  let headers = Headers::new().set("Server", "Knowledge-Server");
  let mut server = Server::with_state(state);
  server.middleware(headers);
  server.at("/").get(tide::redirect("/graphiql"));
  server.at("/").head(handle_root_head);
  server.at("/graphql").post(handle_graphql);
  server.at("/graphiql").get(handle_graphiql);

  server.listen(address).await
}
