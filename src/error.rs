#[derive(Debug, Clone)]
pub enum Error {
    IO(i32),
    Http(httparse::Error),
    ParseInt(core::num::ParseIntError),
}
