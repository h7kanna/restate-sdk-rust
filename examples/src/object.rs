use restate::endpoint;

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    endpoint(service).await
}

#[restate::bundle]
mod bundle {
    use restate::{ContextBase, ObjectContext, ObjectSharedContext};
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

    #[restate::object]
    impl ObjectService {
        const NAME: &'static str = "ObjectService";
        const TYPE: &'static str = "VIRTUAL_OBJECT";

        #[restate::handler]
        pub async fn increment(ctx: ObjectContext, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
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
        pub async fn count(ctx: ObjectSharedContext, name: SignalInput) -> Result<(), anyhow::Error> {
            future::pending::<()>().await;
            Ok(())
        }
    }
}
