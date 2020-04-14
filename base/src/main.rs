pub mod data;
mod loader;
pub mod schema;
pub mod server;
mod store;

use async_std;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    server::activate("127.0.0.1:8080").await
}
