use crate::connection::RestateStreamConsumer;
use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{server::conn::http2, service::service_fn, Request, Response, Result};
use hyper_util::rt::{TokioExecutor, TokioIo};
use prost::Message;
use std::{future::Future, net::SocketAddr};
use tokio::net::TcpListener;
use tracing::info;

pub mod handler;
pub mod http2_handler;
mod service;

// TODO: builder
pub struct RestateEndpointOptions {
    pub listen_address: String,
    pub listen_port: u16,
}

impl Default for RestateEndpointOptions {
    fn default() -> Self {
        Self {
            listen_address: "localhost".to_string(),
            listen_port: 3000,
        }
    }
}

pub struct RestateEndpoint {}

impl RestateEndpoint {
    pub async fn listen<H, F>(
        self,
        options: RestateEndpointOptions,
        handler: H,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        H: Fn(Request<hyper::body::Incoming>) -> F + Send + Clone + 'static,
        F: Future<Output = Result<Response<BoxBody<Bytes, anyhow::Error>>>> + Send + 'static,
    {
        let addr = SocketAddr::from(([127, 0, 0, 1], options.listen_port));

        let listener = TcpListener::bind(addr).await?;
        info!("Listening on http://{}", addr);
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let handler = handler.clone();
            let executor = TokioExecutor::new();
            tokio::task::spawn(async move {
                if let Err(err) = http2::Builder::new(executor)
                    .serve_connection(io, service_fn(handler))
                    .await
                {
                    info!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

pub async fn endpoint<H, F>(
    options: RestateEndpointOptions,
    handler: H,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    H: Fn(Request<hyper::body::Incoming>) -> F + Send + Clone + 'static,
    F: Future<Output = Result<Response<BoxBody<Bytes, anyhow::Error>>>> + Send + 'static,
{
    let endpoint = RestateEndpoint {};
    endpoint.listen(options, handler).await
}
