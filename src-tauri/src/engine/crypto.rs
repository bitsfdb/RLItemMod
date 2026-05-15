use aes::Aes256;
use block_modes::{BlockMode, Ecb};
use block_modes::block_padding::NoPadding;
use minilzo_rs::LZO;
use flate2::read::ZlibDecoder;
use std::io::Read;

type Aes256Ecb = Ecb<Aes256, NoPadding>;

pub fn decrypt_ecb(key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Ecb::new_from_slices(key, &[]).map_err(|e| e.to_string())?;
    let mut buffer = data.to_vec();
    cipher.decrypt(&mut buffer).map_err(|e| e.to_string())?;
    Ok(buffer)
}

pub fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = ZlibDecoder::new(data);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output).map_err(|e| e.to_string())?;
    Ok(output)
}

pub fn decompress_lzo(data: &[u8], uncompressed_size: usize) -> Result<Vec<u8>, String> {
    let lzo = LZO::init().map_err(|e| format!("LZO init failed: {}", e))?;
    let mut output = vec![0u8; uncompressed_size];
    lzo.decompress_safe(data, &mut output).map_err(|e| format!("LZO decompress failed: {}", e))?;
    Ok(output)
}
