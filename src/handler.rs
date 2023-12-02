use crate::Response;
use core::future::Future;

pub trait Handler<'a> {
    type T: Into<Response>;
    fn call(
        self,
        addr: core::net::SocketAddr,
        req: httparse::Request<'a, 'a>,
        body: &'a [u8],
    ) -> impl Future<Output = Self::T> + 'a;
}

impl<'a, F, Fut, T> Handler<'a> for F
where
    T: Into<Response>,
    Fut: Future<Output = T> + 'a,
    F: FnOnce(core::net::SocketAddr, httparse::Request<'a, 'a>, &'a [u8]) -> Fut,
{
    type T = T;

    fn call(
        self,
        addr: core::net::SocketAddr,
        req: httparse::Request<'a, 'a>,
        body: &'a [u8],
    ) -> impl Future<Output = T> + 'a {
        self(addr, req, body)
    }
}
