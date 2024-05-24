use restate_sdk::endpoint::endpoint;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    endpoint().await
}
