use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Bytes;

/// Convenience function to counstruct an empty Request or Response Body
pub fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

/// Convenience function to counstruct Request or Response Body from some chunk of Bytes.
pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
