use restate::{endpoint, RestateEndpointOptions};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // initialize tracing
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "example1=debug,restate_sdk=debug,tower_http=debug".into());
    let replay_filter = restate::logger::ReplayFilter::new();
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_filter(replay_filter))
        .init();
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
    use restate::{Context, ContextBase, JournalIndex};
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

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
            tokio::time::sleep(Duration::from_secs(25)).await;
            Ok(ExecOutput { test: name.test })
        }
    }

    #[restate::service]
    impl Service {
        const NAME: &'static str = "Service";
        const TYPE: &'static str = "SERVICE";

        #[restate::handler]
        pub async fn service(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            let output = ctx.echo_service_client().echo(name.clone());
            let timeout = ctx.timeout(output, 10000);
            let output = timeout.await;
            match output {
                Ok(output) => {
                    Ok(ExecOutput { test: output?.test })
                }
                Err(_) => {
                    Ok(ExecOutput { test: "timeout".to_string() })
                }
            }
        }
    }
}