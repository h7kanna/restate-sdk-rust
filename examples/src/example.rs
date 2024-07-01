use restate::endpoint;

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    endpoint(service).await
}

#[restate::bundle]
mod bundle {
    use restate::{async_recursion, Context, ContextBase};
    use serde::{Deserialize, Serialize};

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
            println!("SimpleService: greet: input {:?}", name);
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
            println!("Service: service: awakeable {:?}", id);
            let output = awakeable.await?;
            println!("Service: service: awakeable received {:?}", output);
            println!("Service: service: input {:?}", name);
            let output = ctx.simple_service_client().greet(name.clone()).await?;
            println!("Service: service: sleeping {:?}", name);
            ctx.sleep(10000).await?;
            println!("Service: service: woke up {:?}", name);
            println!("Simple service output {:?}", output);
            // Calling ourselves
            let output = ctx.service_client().greet(name.clone()).await?;
            println!("Self greet output {:?}", output);
            Ok(ExecOutput { test: output.test })
        }

        #[restate::handler]
        pub async fn greet(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            println!("Service: greet: input {:?}", name);
            Ok(ExecOutput {
                test: format!("success result {}", name.test),
            })
        }
    }
}
