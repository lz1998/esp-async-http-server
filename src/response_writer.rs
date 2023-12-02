use alloc::string::ToString;
use core::future::Future;

pub trait ResponseWriter {
    fn write_all<'a>(&'a self, buf: &'a [u8]) -> impl Future<Output = Result<(), i32>> + 'a;

    fn write_status<'a>(
        &'a self,
        code: u32,
        msg: &'a str,
    ) -> impl Future<Output = Result<(), i32>> + 'a {
        async move {
            self.write_all(b"HTTP/1.1 ").await?;
            self.write_all(code.to_string().as_bytes()).await?;
            self.write_all(b" ").await?;
            self.write_all(msg.as_bytes()).await?;
            self.write_new_line().await
        }
    }

    fn write_header<'a>(
        &'a self,
        name: &'a str,
        value: &'a str,
    ) -> impl Future<Output = Result<(), i32>> + 'a {
        async {
            self.write_all(name.as_bytes()).await?;
            self.write_all(b": ").await?;
            self.write_all(value.as_bytes()).await?;
            self.write_new_line().await
        }
    }

    fn write_new_line(&self) -> impl Future<Output = Result<(), i32>> + '_ {
        self.write_all(b"\r\n")
    }

    fn write_body<'a>(&'a self, body: &'a [u8]) -> impl Future<Output = Result<(), i32>> + 'a {
        async {
            self.write_all(body).await?;
            self.write_new_line().await
        }
    }
}

impl ResponseWriter for esp_async_tcp::TcpStream {
    async fn write_all<'a>(&'a self, buf: &'a [u8]) -> Result<(), i32> {
        let mut offset = 0usize;
        while offset < buf.len() {
            let len = self.write(&buf[offset..]).await? as usize;
            offset += len;
        }
        Ok(())
    }
}
