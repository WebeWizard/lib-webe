//! Streaming chunked transfer-coding encoder for response bodies.
//!
//! [`encode_chunked`] reads a body to end and writes it as HTTP/1.1 `chunked`
//! transfer-coding without buffering the whole body in memory, so unknown-length
//! responses stay streamed.

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::constants::WEBE_BUFFER_SIZE;

/// Streams `reader` to `writer` as `chunked` transfer-coding.
///
/// Each read is emitted as one chunk (`<hex-len>\r\n<data>\r\n`) and the stream
/// is terminated with the final zero-length chunk (`0\r\n\r\n`). The body is read
/// incrementally, never fully buffered.
pub async fn encode_chunked<R, W>(reader: &mut R, writer: &mut W) -> std::io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = [0u8; WEBE_BUFFER_SIZE];
    loop {
        let read = reader.read(&mut buf).await?;
        if read == 0 {
            writer.write_all(b"0\r\n\r\n").await?;
            break;
        }
        let size_line = format!("{read:X}\r\n");
        writer.write_all(size_line.as_bytes()).await?;
        writer.write_all(&buf[..read]).await?;
        writer.write_all(b"\r\n").await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::chunked::ChunkedDecoder;
    use std::io::Cursor;

    #[tokio::test]
    async fn round_trips_through_the_decoder() {
        let data = b"Wikipedia in\r\n\r\nchunks.".to_vec();
        let mut reader = Cursor::new(data.clone());
        let mut encoded: Vec<u8> = Vec::new();
        encode_chunked(&mut reader, &mut encoded).await.unwrap();

        // the encoded stream must end with the terminating zero chunk
        assert!(encoded.ends_with(b"0\r\n\r\n"));

        let mut decoder = ChunkedDecoder::new(Cursor::new(encoded));
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).await.unwrap();
        assert_eq!(decoded, data);
    }

    #[tokio::test]
    async fn empty_body_emits_only_terminator() {
        let mut reader = Cursor::new(Vec::new());
        let mut encoded: Vec<u8> = Vec::new();
        encode_chunked(&mut reader, &mut encoded).await.unwrap();
        assert_eq!(encoded, b"0\r\n\r\n");
    }
}
