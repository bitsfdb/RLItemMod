use aes::Aes256;
use aes::cipher::{BlockDecrypt, KeyInit, generic_array::GenericArray};
use minilzo_rs::LZO;

pub fn decrypt_ecb(key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256::new(GenericArray::from_slice(key));
    let mut buffer = data.to_vec();
    for chunk in buffer.chunks_exact_mut(16) {
        let block = GenericArray::from_mut_slice(chunk);
        cipher.decrypt_block(block);
    }
    Ok(buffer)
}

pub fn decompress_lzo(data: &[u8], uncompressed_size: usize) -> Result<Vec<u8>, String> {
    let lzo = LZO::init().map_err(|e| format!("LZO init failed: {}", e))?;
    let output = lzo.decompress_safe(data, uncompressed_size).map_err(|e| format!("LZO decompress failed: {}", e))?;
    Ok(output)
}