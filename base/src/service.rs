use crate::data::InputResource;
use crate::index::IndexService;
use crate::schema::{Mutations, Schema, State};
use crate::store::DataStore;
use juniper::http::{GraphQLRequest, GraphQLResponse};
use std::io;
use std::sync::Arc;

#[derive(Debug)]
pub struct Service {
    pub schema: Schema,
    pub store: DataStore,
    pub index: Arc<IndexService>,
}
impl Service {
    pub fn new() -> io::Result<Self> {
        let store = DataStore::open()?;
        let index = Arc::new(IndexService::open().unwrap());
        let schema = Schema::new();

        Ok(Service {
            index,
            schema,
            store,
        })
    }
    pub async fn execute<B, F>(&self, request: GraphQLRequest, f: F) -> B
    where
        F: Fn(GraphQLResponse<'_>) -> B,
    {
        let state = State {
            store: self.store.clone(),
            index: self.index.clone(),
        };
        let root = &self.schema.root;
        let response: GraphQLResponse<'_> = request.execute_async(root, &state).await;
        // TODO: Fix no unwrap belongs here.
        state.index.commit().await.unwrap();
        f(response)
    }
    pub async fn ingest(&self, input: InputResource) -> io::Result<()> {
        let state = State {
            store: self.store.clone(),
            index: self.index.clone(),
        };

        Mutations::ingest(&state, input)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.message()))?;

        Ok(())
    }
    pub async fn commit(&self) -> io::Result<()> {
        self.index
            .commit()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;
        Ok(())
    }
}
