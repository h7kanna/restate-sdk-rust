use restate::endpoint;

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    endpoint(service).await
}

#[restate::bundle]
mod bundle {
    use restate::{ContextBase, WorkflowContext};
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

        #[restate::run]
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

        pub async fn pending() -> Result<(), anyhow::Error> {
            future::pending::<()>().await;
            Ok(())
        }
    }
}
