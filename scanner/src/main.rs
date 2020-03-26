mod scanner;
use scanner::activate;

#[async_std::main]
async fn main() {
    activate().await
}
