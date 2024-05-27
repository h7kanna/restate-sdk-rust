use anyhow::Error;
use restate::{
    empty, endpoint::endpoint, full, http2_handler, setup_connection, BodyExt, BoxBody, Bytes, Incoming,
    Method, Request, Response, StatusCode,
};

#[restate::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /*
    let ingress = HttpIngress {};
    let service_client = ingress.simple_service_client();
    let o = service_client.greet(ExecInput { test: "".to_string() }).await?;
    */

    //endpoint_fn(SimpleService);
    //endpoint_fn(Service);
    endpoint(service).await
}

trait ServiceHandler {
    fn name(&self) -> &'static str;
    fn handlers(&self) -> &'static [&'static str];
}

fn endpoint_fn<T: ServiceHandler>(service: T) {
    println!("handlers {:?}", service.handlers());
}

#[restate::bundle]
mod bundle {
    use super::ServiceHandler;
    use restate::{async_recursion, Context};
    use serde::{Deserialize, Serialize};

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
            println!("Service: service: input {:?}", name);
            let output = ctx.simple_service_client().greet(name.clone()).await?;
            println!("Service: service: sleeping {:?}", name);
            //ctx.sleep(5).await?;
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
