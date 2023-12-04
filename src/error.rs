#[derive(Debug, Clone)]
pub enum Error<E: embedded_io_async::Error> {
    IO(E),
    Http(httparse::Error),
    ParseInt(core::num::ParseIntError),
}
