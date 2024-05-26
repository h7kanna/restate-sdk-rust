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

async fn service(req: Request<Incoming>) -> restate::Result<Response<BoxBody<Bytes, Error>>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/discover") => {
            let manifest = r#"{
  "protocolMode": "BIDI_STREAM",
  "minProtocolVersion": 1,
  "maxProtocolVersion": 1,
  "services": [
    {
      "name": "Greeter",
      "ty": "SERVICE",
      "handlers": [
        {
          "name": "greet",
          "ty": "EXCLUSIVE"
        },
        {
          "name": "greet2",
          "ty": "EXCLUSIVE"
        }
      ]
    }
  ]
}"#;
            println!("{}, {}", req.method(), req.uri().path());
            for (name, header) in req.headers() {
                println!("{:?}, {:?}", name, header);
            }

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .header("x-restate-server", "restate-sdk-rust/0.1.0")
                .body(full(manifest).map_err(|e| e.into()).boxed())
                .unwrap();
            Ok(response)
        }

        (&Method::POST, "/invoke/Greeter/greet") => {
            println!("{}, {}", req.method(), req.uri().path());
            for (name, header) in req.headers() {
                println!("{:?}, {:?}", name, header);
            }

            let (http2conn, boxed_body) = Http2Connection::new(req);

            tokio::spawn(
                async move { http2_handler::handle(crate::bundle::Service::service, http2conn).await },
            );

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/restate")
                .header("x-restate-server", "restate-sdk-rust/0.1.0")
                .body(boxed_body)
                .unwrap();

            Ok(response)
        }

        (&Method::POST, "/invoke/Greeter/greet2") => {
            println!("{}, {}", req.method(), req.uri().path());
            for (name, header) in req.headers() {
                println!("{:?}, {:?}", name, header);
            }

            let (http2conn, boxed_body) = Http2Connection::new(req);

            tokio::spawn(async move {
                http2_handler::handle(crate::bundle::SimpleService::greet, http2conn).await
            });

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/restate")
                .header("x-restate-server", "restate-sdk-rust/0.1.0")
                .body(boxed_body)
                .unwrap();
            Ok(response)
        }

        // Return the 404 Not Found for other routes.
        _ => {
            println!("{}, {}", req.method(), req.uri().path());
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(empty().map_err(|e| e.into()).boxed())
                .unwrap();
            Ok(response)
        }
    }
}
