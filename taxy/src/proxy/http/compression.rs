use brotli::CompressorWriter;
use futures::{Stream, StreamExt};
use hyper::{
    body::{Bytes, HttpBody},
    Body,
};
use std::{
    io::Write,
    pin::Pin,
    task::{self, Poll},
};

pub struct CompressionStream {
    body: Body,
    writer: Option<CompressorWriter<Vec<u8>>>,
}

impl CompressionStream {
    pub fn new(body: Body) -> Self {
        let writer = CompressorWriter::new(Vec::new(), 4096, 8, 22);
        Self {
            body,
            writer: Some(writer),
        }
    }
}

impl Stream for CompressionStream {
    type Item = Result<Bytes, hyper::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let poll = self.body.poll_next_unpin(cx);
        if let Some(writer) = &mut self.writer {
            if let Poll::Ready(Some(Ok(chunk))) = &poll {
                let _ = writer.write_all(chunk);
                let _ = writer.flush();
            }
        }
        match poll {
            Poll::Ready(Some(Ok(_))) | Poll::Ready(None) => {
                if let Some(mut writer) = self.writer.take() {
                    if self.body.is_end_stream() {
                        Poll::Ready(Some(Ok(Bytes::from(writer.into_inner()))))
                    } else {
                        let buffer = std::mem::take(writer.get_mut());
                        self.writer = Some(writer);
                        Poll::Ready(Some(Ok(Bytes::from(buffer))))
                    }
                } else {
                    Poll::Ready(None)
                }
            }
            _ => poll,
        }
    }
}
