use restate::{endpoint, RestateEndpointOptions};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // initialize tracing
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "example3=debug,restate_sdk=debug,tower_http=debug".into());
    let replay_filter = restate::logger::ReplayFilter::new();
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_filter(replay_filter))
        .init();
    endpoint(RestateEndpointOptions::default(), service).await
}

#[restate::bundle]
mod bundle {
    use restate::{async_recursion, Context, ContextBase, JournalIndex};
    use serde::{Deserialize, Serialize};
    use std::{future, time::Duration};
    use tracing::info;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExecInput {
        test: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExecOutput {
        test: String,
    }

    #[restate::service]
    impl Service {
        const NAME: &'static str = "Service";
        const TYPE: &'static str = "SERVICE";

        #[async_recursion]
        #[restate::handler]
        pub async fn service(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            ctx.run("step1", move || async move {
                info!("Service: action: doing important work");
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok(())
            })
            .await?;
            ctx.run("step2", move || async move {
                info!("Service: action: doing more important work");
                tokio::time::sleep(Duration::from_secs(20)).await;
                Ok(())
            })
            .await?;
            info!("Service: action: never ending, cleanup");
            ctx.run("pending", Self::pending).await?;
            Ok(ExecOutput { test: name.test })
        }

        pub async fn pending() -> Result<(), anyhow::Error> {
            future::pending::<()>().await;
            Ok(())
        }
    }
}
