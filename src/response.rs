use crate::ResponseWriter;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use esp_async_tcp::TcpStream;

#[derive(Default, Clone, Debug)]
pub struct Response {
    // 200
    pub status: u32,
    // OK
    pub msg: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub async fn write_to(self, stream: &mut TcpStream) -> Result<(), i32> {
        stream.write_status(self.status, &self.msg).await?;
        for (k, v) in self.headers.into_iter() {
            stream.write_header(&k, &v).await?;
        }
        stream.write_new_line().await?;
        stream.write_body(&self.body).await?;
        Ok(())
    }
}

impl From<u32> for Response {
    fn from(value: u32) -> Self {
        let msg = match value {
            200 => "OK".to_string(),
            400 => "Bad Request".to_string(),
            404 => "Not Found".to_string(),
            other => other.to_string(),
        };

        Self {
            msg,
            status: value,
            ..Default::default()
        }
    }
}

impl From<&'static str> for Response {
    fn from(value: &'static str) -> Self {
        Self {
            msg: "OK".to_string(),
            status: 200,
            body: value.as_bytes().to_vec(),
            ..Default::default()
        }
    }
}

impl<T, E> From<Result<T, E>> for Response
where
    T: Into<Response>,
    E: Into<Response>,
{
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(resp) => resp.into(),
            Err(err) => err.into(),
        }
    }
}
// TODO json
