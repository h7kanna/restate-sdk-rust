use restate::endpoint;

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    endpoint(service).await
}

#[restate::bundle]
mod bundle {
    use restate::{
        ContextBase, ContextWorkflowShared, DurablePromise, WorkflowContext, WorkflowSharedContext,
    };
    use serde::{Deserialize, Serialize};
    use std::{future, time::Duration};

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
            let (id, result) = ctx.awakeable::<SignalInput>();
            ctx.run(move || {
                let id = id.clone();
                async move {
                    println!("Service: service: saving signal: {}", id);
                    Ok(())
                }
            })
            .await
            .unwrap();
            let signal_input = result.await.unwrap();
            println!("Signal output: {:?}", signal_input);
            Ok(ExecOutput { test: name.test })
        }

        #[restate::handler]
        pub async fn signal(ctx: WorkflowSharedContext, name: SignalInput) -> Result<(), anyhow::Error> {
            ctx.promise("await_user".to_string()).resolve(Some(name)).await;
            future::pending::<()>().await;
            Ok(())
        }
    }
}
