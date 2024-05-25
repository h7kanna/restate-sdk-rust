use restate::{endpoint::endpoint, Context};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ExecInput {
    test: String,
}

#[derive(Serialize, Deserialize)]
pub struct ExecOutput {
    test: String,
}

async fn service_fn(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
    let output = ctx
        .invoke(greet_fn, "Greeter".to_string(), "greet".to_string(), name, None)
        .await?;
    Ok(ExecOutput { test: output.test })
}

async fn greet_fn(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
    Ok(ExecOutput { test: name.test })
}

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    endpoint().await
}