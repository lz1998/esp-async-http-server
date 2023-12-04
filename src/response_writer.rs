use alloc::string::ToString;
use core::future::Future;
use embedded_io_async::{Read, Write};

pub trait ResponseWriter {
    type Error;
    fn write_all<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a;

    fn write_status<'a>(
        &'a mut self,
        code: u32,
        msg: &'a str,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a {
        async move {
            self.write_all(b"HTTP/1.1 ").await?;
            self.write_all(code.to_string().as_bytes()).await?;
            self.write_all(b" ").await?;
            self.write_all(msg.as_bytes()).await?;
            self.write_new_line().await
        }
    }

    fn write_header<'a>(
        &'a mut self,
        name: &'a str,
        value: &'a str,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a {
        async {
            self.write_all(name.as_bytes()).await?;
            self.write_all(b": ").await?;
            self.write_all(value.as_bytes()).await?;
            self.write_new_line().await
        }
    }

    fn write_new_line(&mut self) -> impl Future<Output = Result<(), Self::Error>> + '_ {
        self.write_all(b"\r\n")
    }

    fn write_body<'a>(
        &'a mut self,
        body: &'a [u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a {
        async {
            self.write_all(body).await?;
            self.write_new_line().await
        }
    }
}

impl<S> ResponseWriter for S
where
    S: Read + Write,
{
    type Error = S::Error;

    async fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> Result<(), Self::Error> {
        let mut offset = 0usize;
        while offset < buf.len() {
            let len = self.write(&buf[offset..]).await?;
            offset += len;
        }
        Ok(())
    }
}
