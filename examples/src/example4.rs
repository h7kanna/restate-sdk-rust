use restate::{endpoint, RestateEndpointOptions};

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    pub struct SignalInput {
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
            let (id, result) = ctx.awakeable::<SignalInput>();
            ctx.run("action", move || {
                let id = id.clone();
                async move {
                    info!("Service: service: saving signal: {}", id);
                    Ok(())
                }
            })
            .await
            .unwrap();
            let signal_input = result.await.unwrap();
            info!("Signal output: {:?}", signal_input);
            Ok(ExecOutput { test: name.test })
        }

        pub async fn pending() -> Result<(), anyhow::Error> {
            future::pending::<()>().await;
            Ok(())
        }
    }
}
