use restate::{endpoint, RestateEndpointOptions};
use tracing::info;

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    endpoint(RestateEndpointOptions::default(), service).await
}

trait ServiceHandler {
    fn name(&self) -> &'static str;
    fn handlers(&self) -> &'static [&'static str];
}

fn endpoint_fn<T: ServiceHandler>(service: T) {
    info!("handlers {:?}", service.handlers());
}

#[restate::bundle]
mod bundle {
    use restate::{Context, ContextBase};
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
