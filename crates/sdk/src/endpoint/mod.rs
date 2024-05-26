use crate::connection::RestateStreamConsumer;
use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{server::conn::http2, service::service_fn, Request, Response, Result};
use hyper_util::rt::{TokioExecutor, TokioIo};
use prost::Message;
use std::{future::Future, net::SocketAddr};
use tokio::net::TcpListener;

pub mod http2_handler;
mod service;

pub struct RestateEndpoint {}

impl RestateEndpoint {
    pub async fn listen<H, F>(
        self,
        handler: H,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        H: Fn(Request<hyper::body::Incoming>) -> F + Send + Clone + 'static,
        F: Future<Output = Result<Response<BoxBody<Bytes, anyhow::Error>>>> + Send + 'static,
    {
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

        let listener = TcpListener::bind(addr).await?;
        println!("Listening on http://{}", addr);
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
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

pub async fn endpoint<H, F>(handler: H) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    H: Fn(Request<hyper::body::Incoming>) -> F + Send + Clone + 'static,
    F: Future<Output = Result<Response<BoxBody<Bytes, anyhow::Error>>>> + Send + 'static,
{
    let endpoint = RestateEndpoint {};
    endpoint.listen(handler).await
}
