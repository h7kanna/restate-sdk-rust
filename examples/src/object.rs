use restate::endpoint;

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    endpoint(service).await
}

#[restate::bundle]
mod bundle {
    use restate::{ContextBase, KeyValueStore, KeyValueStoreReadOnly, ObjectContext, ObjectSharedContext};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CounterInput {
        value: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SignalInput {
        value: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CounterOutput {
        value: String,
    }

    #[restate::object]
    impl ObjectService {
        const NAME: &'static str = "ObjectService";
        const TYPE: &'static str = "VIRTUAL_OBJECT";

        #[restate::handler]
        pub async fn increment(
            ctx: ObjectContext,
            input: CounterInput,
        ) -> Result<CounterOutput, anyhow::Error> {
            ctx.set("count", input.clone()).await;
            Ok(CounterOutput { value: input.value })
        }

        #[restate::handler]
        pub async fn count(ctx: ObjectSharedContext, signal: SignalInput) -> Result<(), anyhow::Error> {
            //let output = ctx.get::<CounterInput, _>("count").await;
            let output: Option<CounterInput> = ctx.get("count").await;
            println!("Printing state: {:?}", output);
            Ok(())
        }
    }
}
