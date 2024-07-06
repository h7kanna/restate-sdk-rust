use restate::endpoint;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // initialize tracing
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "workflow=debug,restate_sdk=info,tower_http=debug".into());
    let replay_filter = restate::logger::ReplayFilter::new();
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_filter(replay_filter))
        .init();
    endpoint(service).await
}

#[restate::bundle]
mod bundle {
    use restate::{
        ContextBase, ContextWorkflowShared, DurablePromise, WorkflowContext, WorkflowSharedContext,
    };
    use serde::{Deserialize, Serialize};
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

    #[restate::workflow]
    impl WorkflowService {
        const NAME: &'static str = "WorkflowService";
        const TYPE: &'static str = "WORKFLOW";

        #[restate::handler]
        pub async fn run(ctx: WorkflowContext, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            let signal_input: SignalInput = ctx.promise("await_user1").awaitable().await;
            info!("Signal1 output: {:?}", signal_input);
            let signal_input: SignalInput = ctx.promise("await_user2").awaitable().await;
            info!("Signal2 output: {:?}", signal_input);
            let signal_input: SignalInput = ctx.promise("await_user3").awaitable().await;
            info!("Signal3 output: {:?}", signal_input);
            Ok(ExecOutput { test: name.test })
        }

        #[restate::handler]
        pub async fn signal(ctx: WorkflowSharedContext, name: SignalInput) -> Result<(), anyhow::Error> {
            //ctx.promise("await_user".to_string()).resolve(Some(name)).await;
            ctx.promise(name.test.clone()).resolve(Some(name)).await;
            Ok(())
        }
    }
}
