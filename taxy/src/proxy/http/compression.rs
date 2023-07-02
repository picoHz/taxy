use brotli::CompressorWriter;
use futures::{Stream, StreamExt};
use hyper::{
    body::{Bytes, HttpBody},
    Body,
};
use phf::phf_map;
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

pub fn is_compressed(content_type: &[u8]) -> bool {
    let content_type = if let Ok(content_type) = std::str::from_utf8(content_type) {
        content_type.to_ascii_lowercase()
    } else {
        return false;
    };
    if let Some(&known) = KNOWN_TYPES.get(&content_type) {
        return known;
    }
    !(content_type.starts_with("text/") || content_type.starts_with("application/"))
}

static KNOWN_TYPES: phf::Map<&'static str, bool> = phf_map! {
    "image/svg+xml" => false,
    "image/bmp" => false,
    "image/x-ms-bmp" => false,
    "audio/wav" => false,
    "audio/x-wav" => false,
    "audio/midi" => false,
    "audio/x-midi" => false,
    "application/x-bzip" => true,
    "application/x-bzip2" => true,
    "application/gzip" => true,
    "application/vnd.rar" => true,
    "application/x-tar" => true,
    "application/zip" => true,
    "application/x-7z-compressed" => true,
    "application/epub+zip" => true,
    "font/otf" => false,
    "font/ttf" => false,
};

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_is_compressed() {
        assert!(!is_compressed(b"text/html"));
        assert!(!is_compressed(b"application/json"));
        assert!(!is_compressed(b"image/svg+xml"));
        assert!(!is_compressed(b"image/bmp"));
        assert!(is_compressed(b"image/png"));
        assert!(!is_compressed(b"image/x-ms-bmp"));
        assert!(is_compressed(b"audio/mp3"));
        assert!(!is_compressed(b"audio/wav"));
        assert!(!is_compressed(b"audio/x-wav"));
        assert!(!is_compressed(b"audio/midi"));
        assert!(!is_compressed(b"audio/x-midi"));
        assert!(is_compressed(b"video/webm"));
        assert!(is_compressed(b"application/x-bzip"));
        assert!(is_compressed(b"application/x-bzip2"));
        assert!(is_compressed(b"application/gzip"));
    }
}
