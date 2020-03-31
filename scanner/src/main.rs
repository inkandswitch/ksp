mod markdown;
mod resource;
mod scanner;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    scanner::activate().await
}
