use restate::endpoint;

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    endpoint(service).await
}

#[restate::bundle]
mod bundle {
    use restate::{async_recursion, Context, ContextBase};
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
            ctx.run(move || async move {
                info!("Service: service: sleeping");
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok(())
            })
            .await?;
            info!("Service: run: pending {:?}", name);
            ctx.run(Self::pending).await?;
            Ok(ExecOutput { test: name.test })
        }

        pub async fn pending() -> Result<(), anyhow::Error> {
            future::pending::<()>().await;
            Ok(())
        }
    }
}
