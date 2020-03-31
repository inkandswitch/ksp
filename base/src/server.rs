use crate::schema::{schema, State};
use async_std::task;
use tide::{Request, Response, Server};

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

pub async fn activate() -> std::io::Result<()> {
  let state = crate::schema::init()?;
  let mut service = Server::with_state(state);
  service.at("/").get(tide::redirect("/graphiql"));
  service.at("/graphql").post(handle_graphql);
  service.at("/graphiql").get(handle_graphiql);
  service.listen("0.0.0.0:8080").await?;
  Ok(())
}
