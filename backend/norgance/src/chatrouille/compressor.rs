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
  let mut encoder = libflate::zlib::Encoder::new(Vec::new()).context(EncoderError)?;

  std::io::copy(&mut &data[..], &mut encoder).context(EncoderError)?;

  let compressed_data = encoder.finish().into_result().context(EncoderError)?;
  Ok(compressed_data)
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
  use std::io::Read;
  let mut decoder = libflate::zlib::Decoder::new(&data[..]).context(DecoderError)?;

  let mut decoded_data = Vec::new();
  decoder
    .read_to_end(&mut decoded_data)
    .context(DecoderError)?;

  Ok(decoded_data)
}
