const NEW_LINE: [u8; 2] = [b'\r', b'\n'];
use std::io::{BufRead, Error, Read, Write, Cursor};
use std::marker::Unpin;
use std::pin::Pin;

use pin_project_lite::pin_project;

use tokio::io::{AsyncBufRead, AsyncRead, ReadBuf};
use std::task::{Context, Poll};

// Must use 'where' style trait bounds because of this bug: https://github.com/taiki-e/pin-project-lite/issues/2
pin_project! {
  pub struct ChunkedDecoder<B>
  where
    B: AsyncBufRead,
    B: Unpin,
  {
    finished: bool,
    cur_chunk_size: usize,
    cur_chunk_pos: usize,
    #[pin]
    inner: B,
  }
}

impl<B: AsyncBufRead+Unpin> ChunkedDecoder<B> {
  pub fn new(inner: B) -> ChunkedDecoder<B> {
    ChunkedDecoder {
      finished: false,
      cur_chunk_size: 0,
      cur_chunk_pos: 0,
      inner: inner,
    }
  }
}

impl<B: AsyncBufRead+Unpin> AsyncRead for ChunkedDecoder<B> {
  fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<tokio::io::Result<()>> {
    let mut this = self.project();
    if *this.finished {
      return Poll::Ready(Ok(())); // already finished with all chunks
    }

    let mut inner_buf: Cursor<Vec<u8>>;
    
    match this.inner.as_mut().poll_fill_buf(cx) {
      Poll::Ready(Ok(data)) => inner_buf = Cursor::new(Vec::from(data)), // TODO: I hate this, but it's the only way I can find to drop the 'this.inner' borrow.
      Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
      Poll::Pending => return Poll::Pending
    }

    // if start of chunk - read in the new chunk size
    if *this.cur_chunk_size == 0 && *this.cur_chunk_pos == 0 {
      let mut chunk_size_line = String::new();
      match inner_buf.read_line(&mut chunk_size_line) {
        Ok(size) => {
          this.inner.as_mut().consume(size);
          // parse chunk size from hex string
          match usize::from_str_radix(&chunk_size_line[..(size - 2)], 16) {
            Ok(chunk_size) => {
              if chunk_size == 0 {
                // completely finished with all chunks
                *this.finished = true;
                return Poll::Ready(Ok(()));
              } else {
                *this.cur_chunk_size = chunk_size;
              }
            }
            Err(_error) => return Poll::Ready(Err(std::io::Error::from(std::io::ErrorKind::InvalidData))),
          }
        }
        Err(error) => return Poll::Ready(Err(error)),
      }
    }
    // read from remaining chunk data

    let limit = std::cmp::min(
      *this.cur_chunk_size - *this.cur_chunk_pos,
      buf.remaining()
    );

    let mut take = inner_buf.take(limit as u64);
    let mut temp = Vec::from(buf.initialize_unfilled()); // TODO: not sure why I can't read directly into buf
    match take.read(&mut temp) {
      Ok(size) => {
        temp.truncate(size);
        this.inner.as_mut().consume(size);
        *this.cur_chunk_pos += size;
        if *this.cur_chunk_pos == *this.cur_chunk_size {
          // at the end of the current chunk
          // trash the ending new line chars
          this.inner.as_mut().consume(NEW_LINE.len());
          // reset self
          *this.cur_chunk_pos = 0;
          *this.cur_chunk_size = 0;
          buf.put_slice(&temp);
        }
        return Poll::Ready(Ok(()));
      }
      Err(error) => return Poll::Ready(Err(error)),
    }
  }
}

pub struct ChunkedEncoder<W> {
  finished: bool,
  inner: W,
}

impl<W: Write> ChunkedEncoder<W> {
  pub fn new(inner: W) -> ChunkedEncoder<W> {
    ChunkedEncoder {
      finished: false,
      inner: inner,
    }
  }
}

impl<W: Write> Write for ChunkedEncoder<W> {
  fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
    // NOTE:  it may be a good idea to not 'finish' the writer until a finish() method is called.
    // ----- in case whatever is calling this write is just naively passing data from another source.
    // if writer is finished, return 0
    if self.finished {
      return Ok(0);
    }
    // if length of buf is 0, mark the writer as finished
    let hex_len_line = format!("{:X}\r\n", buf.len());
    match self.inner.write_all(&hex_len_line.as_bytes()) {
      Ok(()) => {
        // write the chunk data
        match self.inner.write_all(buf) {
          Ok(()) => {
            // write the chunk newline
            match self.inner.write_all(b"\r\n") {
              Ok(()) => {
                // if buf is empty, then mark writer as finished
                if buf.len() == 0 {
                  self.finished = true;
                }
                return Ok(hex_len_line.len() + buf.len() + 2);
              }
              Err(error) => return Err(error),
            }
          }
          Err(error) => return Err(error),
        }
      }
      Err(error) => return Err(error),
    }
  }

  fn flush(&mut self) -> Result<(), Error> {
    return self.inner.flush();
  }
}

// In the following example, three chunks of length 4, 5 and 14 (hexadecimal "E") are shown. The chunk size is transferred as a hexadecimal number followed by \r\n as a line separator, followed by a chunk of data of the given size.

// 4\r\n
// Wiki\r\n
// 5\r\n
// pedia\r\n
// E\r\n
//  in\r\n
// \r\n
// chunks.\r\n
// 0\r\n
// \r\n

#[tokio::test]
async fn chunked_decoder() {
  use std::io::Cursor;
  use tokio::io::{AsyncReadExt};

  let chunked_data = "4\r\nWiki\r\n5\r\npedia\r\nE\r\n in\r\n\r\nchunks.\r\n0\r\n\r\n".as_bytes();
  let mut chunk_reader = ChunkedDecoder::new(Cursor::new(chunked_data));
  let mut decoded = String::new();
  chunk_reader.read_to_string(&mut decoded).await.unwrap();
  assert_eq!(decoded, "Wikipedia in\r\n\r\nchunks.".to_owned());
}

#[tokio::test]
async fn chunked_encoder() {
  let buf = Vec::new();
  let mut chunked_writer = ChunkedEncoder::new(buf);
  chunked_writer.write(b"Wiki").unwrap();
  chunked_writer.write(b"pedia").unwrap();
  chunked_writer.write(b" in\r\n\r\nchunks.").unwrap();
  chunked_writer.write(b"").unwrap(); // ending chunk
  assert_eq!(
    &chunked_writer.inner.as_slice(),
    &"4\r\nWiki\r\n5\r\npedia\r\nE\r\n in\r\n\r\nchunks.\r\n0\r\n\r\n".as_bytes()
  );
}
