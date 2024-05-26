#[restate::bundle]
mod handler {
    use restate::Context;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExecInput {
        test: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExecOutput {
        test: String,
    }

    #[restate::service]
    impl EchoService {
        const NAME: &'static str = "Echo";
        const TYPE: &'static str = "SERVICE";

        #[restate::handler]
        pub async fn echo(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            Ok(ExecOutput { test: name.test })
        }
    }

    #[restate::service]
    impl Service {
        const NAME: &'static str = "Service";
        const TYPE: &'static str = "SERVICE";

        #[restate::handler]
        pub async fn service(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            let output = ctx.echo_service_client().echo(name.clone()).await?;
            Ok(ExecOutput { test: output.test })
        }
    }
}

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    restate::endpoint::endpoint(service).await
}
