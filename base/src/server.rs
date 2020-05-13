use crate::service::Service;
use async_trait::async_trait;
use futures::future::BoxFuture;
use juniper::http::GraphQLRequest;
use log;
use std::sync::RwLock;
use tide::{Middleware, Next, Request, Response, Server};

// #[derive(Debug)]
// struct State {
//     pub schema: Schema,
//     pub state: SchemaState,
// }
type State = Service;

async fn handle_graphiql(_: Request<State>) -> Response {
    Response::new(200)
        .body_string(juniper::http::graphiql::graphiql_source("/graphql"))
        .set_header("content-type", "text/html;charset=utf-8")
}

async fn handle_graphql(mut request: Request<State>) -> Response {
    log::info!("Received graphql query");
    let json = request.body_json().await;

    let query: GraphQLRequest = json.expect("be able to deserialize the graphql request");

    // TODO: We had to clone here because otherwise data loader would be shared
    // accross multiple requests which is undesired as:
    // 1. It would use more and more memory over time.
    // 2. DB changes would not be picked up.
    // let state = &request.state().state;
    // let schema = &request.state().schema;
    // let root = &schema.root;

    // // probably worth making the schema a singleton using lazy_static library
    // let response = query.execute_async(root, state).await;
    let state = request.state();
    let response = state
        .execute(query, |response| {
            Response::new(if response.is_ok() { 200 } else { 400 })
                .body_json(&response)
                .expect("be able to serialize the graphql response")
        })
        .await;
    // let status = if response.is_ok() { 200 } else { 400 };
    log::info!("Responding to the graphl query");

    // Response::new(status)
    //     .body_json(&response)
    //     .expect("be able to serialize the graphql response")
    response
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
    // let schema = Schema::new();
    // let state = State {
    //     schema,
    //     state: SchemaState::new()?,
    // };
    let state = Service::new()?;
    let headers = Headers::new().set("Server", "Knowledge-Server");
    let mut server = Server::with_state(state);
    server.middleware(headers);
    server.at("/").get(tide::redirect("/graphiql"));
    server.at("/").head(handle_root_head);
    server.at("/graphql").post(handle_graphql);
    server.at("/graphiql").get(handle_graphiql);

    server.listen(address).await
}
