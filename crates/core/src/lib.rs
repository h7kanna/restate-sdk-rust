//! Restate Rust SDK Core API

use std::future::Future;

pub trait RestateContext {}

pub trait RestateObjectContext {}

pub trait RestateObjectSharedContext {}

/// Trait to represent a run action
pub trait RunAction: Fn() -> Self::OutputFuture {
    /// Output type of the async function
    type Output;
    /// Future of the output
    type OutputFuture: Future<Output = <Self as RunAction>::Output> + Send + 'static;
}

impl<F: ?Sized, Fut> RunAction for F
where
    F: Fn() -> Fut,
    Fut: Future + Send + 'static,
{
    type Output = Fut::Output;
    type OutputFuture = Fut;
}

/// Trait to represent a service handler
pub trait ServiceHandler<Arg0, Arg1>: Fn(Arg0, Arg1) -> Self::OutputFuture {
    /// Output type of the async function
    type Output;
    /// Future of the output
    type OutputFuture: Future<Output = <Self as ServiceHandler<Arg0, Arg1>>::Output> + Send + 'static;
}

impl<F: ?Sized, Fut, Arg0, Arg1> ServiceHandler<Arg0, Arg1> for F
where
    F: Fn(Arg0, Arg1) -> Fut,
    Fut: Future + Send + 'static,
{
    type Output = Fut::Output;
    type OutputFuture = Fut;
}


pub trait Service<Request> {
    /// Responses given by the service.
    type Response;

    /// Errors produced by the service.
    type Error;

    /// The future response value.
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    /// Process the request and return the response asynchronously.
    /// `call` takes `&self` instead of `mut &self` because:
    /// - It prepares the way for async fn,
    ///   since then the future only borrows `&self`, and thus a Service can concurrently handle
    ///   multiple outstanding requests at once.
    /// - It's clearer that Services can likely be cloned
    /// - To share state across clones, you generally need `Arc<Mutex<_>>`
    ///   That means you're not really using the `&mut self` and could do with a `&self`.
    /// The discussion on this is here: <https://github.com/hyperium/hyper/issues/3040>
    fn call(&self, req: Request) -> Self::Future;
}
