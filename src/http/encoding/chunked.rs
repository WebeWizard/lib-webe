use std::io::{BufRead, Error, Read};

pub struct ChunkedDecoder<B: BufRead> {
  finished: bool,
  cur_chunk_size: usize,
  cur_chunk_pos: usize,
  inner: B,
}

impl<B: BufRead> ChunkedDecoder<B> {
  pub fn new(inner: B) -> ChunkedDecoder<B> {
    ChunkedDecoder {
      finished: false,
      cur_chunk_size: 0,
      cur_chunk_pos: 0,
      inner: inner,
    }
  }
}

impl<B: BufRead> Read for ChunkedDecoder<B> {
  fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
    if self.finished {
      // already finished with all chunks
      return Ok(0);
    }
    if self.cur_chunk_size == 0 && self.cur_chunk_pos == 0 {
      // start of chunk - read in the new chunk size
      let mut chunk_size_line = String::new();
      match self.inner.read_line(&mut chunk_size_line) {
        Ok(_size) => {
          // pop off the ending new line chars
          if chunk_size_line.as_bytes().last() == Some(&b'\n') {
            chunk_size_line.pop();
          }
          if chunk_size_line.as_bytes().last() == Some(&b'\r') {
            chunk_size_line.pop();
          }
          // parse chunk size from hex string
          match usize::from_str_radix(chunk_size_line.as_str(), 16) {
            Ok(chunk_size) => {
              if chunk_size == 0 {
                // completely finished with all chunks
                self.finished = true;
                return Ok(0);
              } else {
                self.cur_chunk_size = chunk_size;
              }
            }
            Err(_error) => return Err(std::io::Error::from(std::io::ErrorKind::InvalidData)),
          }
        }
        Err(error) => return Err(error),
      }
    }
    // read from remaining chunk data
    let limit = std::cmp::min(
      self.cur_chunk_size - self.cur_chunk_pos, // read the rest of the current chunk
      buf.len(),                                // fill the rest of the buffer
    );
    let mut take = self.inner.by_ref().take(limit as u64);
    match take.read(buf) {
      Ok(size) => {
        self.cur_chunk_pos += size;
        if self.cur_chunk_pos == self.cur_chunk_size {
          // at the end of the current chunk
          // trash the ending new line chars
          let mut trash = String::with_capacity(2);
          match self.inner.read_line(&mut trash) {
            Ok(_size) => {
              // reset self
              self.cur_chunk_pos = 0;
              self.cur_chunk_size = 0;
            }
            Err(error) => return Err(error),
          }
        }
        return Ok(size);
      }
      Err(error) => return Err(error),
    }
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

#[test]
fn chunked_decoder() {
  use std::io::BufReader;

  let chunked_data = "4\r\nWiki\r\n5\r\npedia\r\nE\r\n in\r\n\r\nchunks.\r\n0\r\n\r\n".as_bytes();
  let raw_reader = BufReader::new(chunked_data);
  let chunk_reader = ChunkedDecoder::new(raw_reader);
  let mut buf_chunk_reader = BufReader::new(chunk_reader);
  let mut decoded = String::new();
  buf_chunk_reader.read_to_string(&mut decoded).unwrap();
  assert_eq!(decoded, "Wikipedia in\r\n\r\nchunks.".to_owned());
}
