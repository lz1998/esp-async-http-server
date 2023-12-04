#![feature(return_position_impl_trait_in_trait)]
#![feature(ip_in_core)]
#![feature(async_fn_in_trait)]
#![no_std]

mod error;
mod handler;
mod response;
mod response_writer;

extern crate alloc;

pub use crate::handler::Handler;
use alloc::string::String;
use alloc::vec::Vec;
use core::net::SocketAddr;
use embedded_io_async::{Read, Write};
pub use error::Error;
pub use httparse::Request;
use httparse::Status;
pub use response::Response;
pub use response_writer::ResponseWriter;

// pub async fn serve<H>(listener: TcpListener, handler: H) -> Result<(), Error>
// where
//     H: for<'a> Handler<'a> + Clone,
// {
//     loop {
//         let (mut stream, addr) = listener.accept().await.map_err(Error::IO)?;
//         if let Err(err) = process_req(&mut stream, addr, handler.clone()).await {
//             log::error!("failed to process req: {err:?}");
//         }
//     }
// }

pub async fn process_req<S, H>(
    stream: &mut S,
    addr: SocketAddr,
    handler: H,
) -> Result<(), Error<S::Error>>
where
    S: Read + Write,
    H: for<'a> Handler<'a>,
{
    let mut req_buf = Vec::with_capacity(1024);
    // read to end
    let (_body_start, _content_length) = loop {
        let mut buf = [0u8; 256];
        let len = stream.read(&mut buf).await.map_err(Error::IO)? as usize;
        req_buf.extend_from_slice(&buf[..len]);
        let mut headers = [httparse::EMPTY_HEADER; 32];
        let mut req = httparse::Request::new(&mut headers);
        match req.parse(&req_buf).map_err(Error::Http)? {
            Status::Complete(body_start) => {
                let content_length = match req
                    .headers
                    .iter()
                    .find(|h| &h.name.to_lowercase() == "content-length")
                {
                    None => None,
                    Some(h) => Some(
                        String::from_utf8_lossy(h.value)
                            .parse::<usize>()
                            .map_err(Error::ParseInt)?,
                    ),
                };
                if let Some(body_len) = content_length.map(|len| len + body_start - req_buf.len()) {
                    while req_buf.len() < body_start + body_len {
                        let mut buf = [0u8; 256];
                        let len = stream.read(&mut buf).await.map_err(Error::IO)? as usize;
                        req_buf.extend_from_slice(&buf[..len]);
                    }
                }
                break (body_start, content_length);
            }
            Status::Partial => continue,
        }
    };
    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut req = httparse::Request::new(&mut headers);
    let body_start = match req.parse(&req_buf).map_err(Error::Http)? {
        Status::Complete(len) => len,
        Status::Partial => unreachable!(),
    };
    handler
        .call(addr, req, &req_buf[body_start..])
        .await
        .into()
        .write_to(stream)
        .await
        .map_err(Error::IO)?;
    Ok(())
}
