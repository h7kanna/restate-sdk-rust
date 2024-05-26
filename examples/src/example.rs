use anyhow::Error;
use restate::{
    empty, endpoint::endpoint, full, http2_handler, BodyExt, BoxBody, Bytes, Http2Connection, Incoming,
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
    use restate::{async_recursion, Context, HttpIngress};
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
            Ok(ExecOutput { test: name.test })
        }
    }

    trait SimpleServiceHandlerClient {
        async fn greet(ctx: &Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error>;
    }

    struct SimpleServiceHandlerClientImpl;

    impl SimpleServiceHandlerClientImpl {
        async fn greet(&self, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            Ok(ExecOutput { test: "".to_string() })
        }
    }

    trait SimpleServiceHandlerClientExt {
        fn simple_service_client(&self) -> SimpleServiceHandlerClientImpl;
    }

    impl SimpleServiceHandlerClientExt for HttpIngress {
        fn simple_service_client(&self) -> SimpleServiceHandlerClientImpl {
            SimpleServiceHandlerClientImpl {}
        }
    }

    #[restate::service]
    impl Service {
        const NAME: &'static str = "Service";
        const TYPE: &'static str = "SERVICE";

        #[async_recursion]
        #[restate::handler]
        pub async fn service(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            println!("I'm here ---------> {:?}", name);
            let output = ctx.greet(name.clone()).await?;
            Ok(ExecOutput { test: output.test })
        }

        #[restate::handler]
        pub async fn greet(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            Ok(ExecOutput { test: name.test })
        }
    }

    trait ServiceExt {
        async fn service(&self, name: ExecInput) -> Result<ExecOutput, anyhow::Error>;
        async fn greet(&self, name: ExecInput) -> Result<ExecOutput, anyhow::Error>;
    }

    impl ServiceExt for Context {
        async fn service(&self, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            self.invoke(
                Service::service,
                "Greeter".to_string(),
                "greet".to_string(),
                name,
                None,
            )
            .await
        }

        async fn greet(&self, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
            self.invoke(
                Service::greet,
                "Greeter".to_string(),
                "greet2".to_string(),
                name,
                None,
            )
            .await
        }
    }

    trait ServiceHandlerClient {
        fn service_client(&self) -> Service;
    }

    impl ServiceHandlerClient for HttpIngress {
        fn service_client(&self) -> Service {
            todo!()
        }
    }
}
