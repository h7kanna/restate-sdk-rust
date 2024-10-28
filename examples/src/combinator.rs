use restate_sdk_api::{self as restate, endpoint, RestateEndpointOptions};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // initialize tracing
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "example1=debug,restate_sdk=info,tower_http=debug".into());
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
    use futures::{join, pin_mut, select, FutureExt};
    use restate_sdk_api::{self as restate, Context, ContextBase};
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    use tracing::info;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EchoInput {
        test: String,
        delay: u64,
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
    impl EchoService {
        const NAME: &'static str = "Echo";
        const TYPE: &'static str = "SERVICE";

        #[restate::handler]
        pub async fn echo(ctx: Context, name: EchoInput) -> Result<ExecOutput, anyhow::Error> {
            tokio::time::sleep(Duration::from_secs(name.delay)).await;
            Ok(ExecOutput { test: name.test })
        }
    }

    #[restate::service]
    impl Service {
        const NAME: &'static str = "Service";
        const TYPE: &'static str = "SERVICE";

        #[restate::handler]
        pub async fn join(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            let output1 = ctx.echo_service_client().echo(EchoInput {
                test: name.test.clone(),
                delay: 20,
            });
            let output2 = ctx.echo_service_client().echo(EchoInput {
                test: name.test.clone(),
                delay: 25,
            });
            // TODO: Placeholder, implement restate::join combinator
            let (output1, output2) = join!(output1, output2);
            Ok(ExecOutput { test: output2?.test })
        }

        #[restate::handler]
        pub async fn select(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            let output1 = ctx
                .echo_service_client()
                .echo(EchoInput {
                    test: name.test.clone(),
                    delay: 15,
                })
                .fuse();
            let output2 = ctx
                .echo_service_client()
                .echo(EchoInput {
                    test: name.test.clone(),
                    delay: 10,
                })
                .fuse();
            pin_mut!(output1, output2);
            // TODO: Placeholder, implement restate::select combinator
            select! {
                output1 = output1 => {
                    info!("echo1 completed first");
                    Ok(ExecOutput { test: output1?.test })
                },
                output2 = output2 => {
                    info!("echo2 completed first");
                   Ok(ExecOutput { test: output2?.test })
                },
            }
        }
    }
}
