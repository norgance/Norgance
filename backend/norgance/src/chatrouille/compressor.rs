use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum CompressorError {
  #[snafu(display("EncoderError: {}", source))]
  EncoderError { source: std::io::Error },
  #[snafu(display("DecoderError: {}", source))]
  DecoderError { source: std::io::Error },
}

pub type Result<T, E = CompressorError> = std::result::Result<T, E>;

pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
  let mut encoder = libflate::deflate::Encoder::new(Vec::new());

  std::io::copy(&mut &data[..], &mut encoder).context(EncoderError)?;

  let compressed_data = encoder.finish().into_result().context(EncoderError)?;
  Ok(compressed_data)
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
  use std::io::Read;
  let mut decoder = libflate::deflate::Decoder::new(&data[..]);

  let mut decoded_data = Vec::new();
  decoder
    .read_to_end(&mut decoded_data)
    .context(DecoderError)?;

  Ok(decoded_data)
}

#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_and_decompress() {
      let data = b"data data data data data data";
      let compressed = match compress(data) {
        Ok(c) => c,
        Err(_) => panic!("compress fail"),
      };
      assert!(data.len() > compressed.len());
      let uncompressed = match decompress(&compressed) {
        Ok(u) => u,
        Err(_) => panic!("decompress fail"),
      };
      assert_eq!(uncompressed, data);
    }
  }