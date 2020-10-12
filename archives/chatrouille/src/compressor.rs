pub fn compress(data: Vec<u8>) -> Option<Vec<u8>> {
  let mut encoder = match libflate::zlib::Encoder::new(Vec::new()) {
    Ok(encoder) => encoder,
    Err(_) => return None,
  };

  match std::io::copy(&mut &data[..], &mut encoder) {
    Ok(_) => {}
    Err(_) => return None,
  }

  match encoder.finish().as_result() {
    Ok(compressed_data) => Some(compressed_data.to_owned()),
    Err(_) => return None,
  }
}

pub fn decompress(data: Vec<u8>) -> Option<Vec<u8>> {
  use std::io::Read;
  let mut decoder = match libflate::zlib::Decoder::new(&data[..]) {
    Ok(decoder) => decoder,
    Err(_) => return None,
  };

  let mut decoded_data = Vec::new();
  match decoder.read_to_end(&mut decoded_data) {
    Ok(_) => {}
    Err(_) => return None,
  }

  return Some(decoded_data);
}
