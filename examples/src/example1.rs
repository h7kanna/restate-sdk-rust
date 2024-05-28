use restate::endpoint::endpoint;
/*
use restate::{
    empty, endpoint::endpoint, full, http2_handler, BodyExt, BoxBody, Bytes, Http2Receiver, Incoming, Method,
    Request, Response, StatusCode,
};
*/

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

    pub struct SimpleService;
    impl SimpleService {
        const NAME: &'static str = "SimpleService";
        const TYPE: &'static str = "SERVICE";

        #[restate::handler]
        pub async fn greet(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            Ok(ExecOutput { test: name.test })
        }
    }

    trait SimpleServiceHandlerClient {
        async fn greet(self, name: ExecInput) -> Result<ExecOutput, anyhow::Error>;
    }

    struct SimpleServiceClientImpl<'a> {
        ctx: &'a Context,
    }

    impl<'a> SimpleServiceClientImpl<'a> {
        async fn greet(self, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            self.ctx
                .invoke(
                    SimpleService::greet,
                    "SimpleService".to_string(),
                    "greet".to_string(),
                    name,
                    None,
                )
                .await
        }
    }

    trait SimpleServiceClientExt {
        fn simple_service_client(&self) -> SimpleServiceClientImpl;
    }

    impl SimpleServiceClientExt for Context {
        fn simple_service_client(&self) -> SimpleServiceClientImpl {
            SimpleServiceClientImpl { ctx: &self }
        }
    }

    pub struct Service;
    impl Service {
        const NAME: &'static str = "Service";
        const TYPE: &'static str = "SERVICE";

        #[async_recursion]
        #[restate::handler]
        pub async fn service(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            let output = ctx.simple_service_client().greet(name.clone()).await?;
            // Calling ourselves
            let output = ctx.service_client().greet(name.clone()).await?;
            Ok(ExecOutput { test: output.test })
        }

        #[restate::handler]
        pub async fn greet(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            Ok(ExecOutput { test: name.test })
        }
    }

    trait ServiceHandlerClient {
        async fn service(self, name: ExecInput) -> Result<ExecOutput, anyhow::Error>;
        async fn greet(self, name: ExecInput) -> Result<ExecOutput, anyhow::Error>;
    }

    struct ServiceClientImpl<'a> {
        ctx: &'a Context,
    }

    impl<'a> ServiceClientImpl<'a> {
        async fn service(self, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            self.ctx
                .invoke(
                    Service::service,
                    "Service".to_string(),
                    "service".to_string(),
                    name,
                    None,
                )
                .await
        }

        async fn greet(self, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            self.ctx
                .invoke(
                    Service::greet,
                    "Service".to_string(),
                    "greet".to_string(),
                    name,
                    None,
                )
                .await
        }
    }

    trait ServiceClientExt {
        fn service_client(&self) -> ServiceClientImpl;
    }

    impl ServiceClientExt for Context {
        fn service_client(&self) -> ServiceClientImpl {
            ServiceClientImpl { ctx: &self }
        }
    }
}
