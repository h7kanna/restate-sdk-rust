use restate::endpoint;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // initialize tracing
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "example5=debug,restate_sdk=info,tower_http=debug".into());
    let replay_filter = restate::logger::ReplayFilter::new();
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_filter(replay_filter))
        .init();
    endpoint(service).await
}

#[restate::bundle]
mod bundle {
    use restate::{async_recursion, Context, ContextBase};
    use serde::{Deserialize, Serialize};
    use tracing::info;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AwakeOutput {
        test: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExecInput {
        test: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExecOutput {
        test: String,
    }

    #[restate::service]
    impl SimpleService {
        const NAME: &'static str = "SimpleService";
        const TYPE: &'static str = "SERVICE";

        #[restate::handler]
        pub async fn greet(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            info!("SimpleService: greet: input {:?}", name);
            Ok(ExecOutput { test: name.test })
        }
    }

    #[restate::service]
    impl Service {
        const NAME: &'static str = "Service";
        const TYPE: &'static str = "SERVICE";

        #[async_recursion]
        #[restate::handler]
        pub async fn service(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            let (id, awakeable) = ctx.awakeable::<AwakeOutput>();
            info!("Service: service: awakeable {:?}", id);
            let output = awakeable.await?;
            info!("Service: service: awakeable received {:?}", output);
            info!("Service: service: input {:?}", name);
            let output = ctx.simple_service_client().greet(name.clone()).await?;
            info!("Service: service: sleeping {:?}", name);
            ctx.sleep(10000).await?;
            info!("Service: service: woke up {:?}", name);
            info!("Simple service output {:?}", output);
            // Calling ourselves
            let output = ctx.service_client().greet(name.clone()).await?;
            info!("Self greet output {:?}", output);
            Ok(ExecOutput { test: output.test })
        }

        #[restate::handler]
        pub async fn greet(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            info!("Service: greet: input {:?}", name);
            Ok(ExecOutput {
                test: format!("success result {}", name.test),
            })
        }
    }
}
