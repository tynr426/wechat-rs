use std::io::Cursor;

use rand::thread_rng;
use rand::Rng;
use base64;
use byteorder::{NativeEndian, WriteBytesExt, ReadBytesExt};
use crypto::buffer::{BufferResult, ReadBuffer, WriteBuffer};
use crypto::{aes, blockmodes, buffer, symmetriccipher};

use types::WeChatResult;
use errors::WeChatError;


#[derive(Debug, Eq, PartialEq)]
pub struct PrpCrypto {
    key: Vec<u8>,
}

impl PrpCrypto {
    pub fn new(key: &[u8]) -> PrpCrypto {
        PrpCrypto {
            key: key.to_vec(),
        }
    }

    fn get_random_string() -> String {
        if cfg!(test) {
            "1234567890123456".to_owned()
        } else {
            thread_rng().gen_ascii_chars().take(16).collect()
        }
    }

    pub fn encrypt(&self, plaintext: &str, _id: &str) -> WeChatResult<String> {
        let mut wtr = PrpCrypto::get_random_string().into_bytes();
        wtr.write_u32::<NativeEndian>((plaintext.len() as u32).to_be()).unwrap();
        wtr.extend(plaintext.bytes());
        wtr.extend(_id.bytes());
        //aes 加密

        // TODO: do not unwrap
        let encrypted = aes256_cbc_encrypt(&wtr.to_vec(), &self.key, &self.key[..16]).unwrap();
        let b64encoded = base64::encode(&encrypted);
        Ok(b64encoded)
    }

    pub fn decrypt(&self, ciphertext: &str, _id: &str) -> WeChatResult<String> {
        let b64decoded = base64::decode(ciphertext)?;
        // TODO: do not unwrap
        let text = aes256_cbc_decrypt(&b64decoded,&self.key, &self.key[..16]).unwrap();
        let mut rdr = Cursor::new(text[16..20].to_vec());
        let content_length = u32::from_be(rdr.read_u32::<NativeEndian>().unwrap()) as usize;
        let content = &text[20 .. content_length + 20];
        let from_id = &text[content_length + 20 ..];
        if from_id != _id.as_bytes() {
            return Err(WeChatError::InvalidAppId);
        }
        let content_string = String::from_utf8(content.to_vec()).unwrap();
        Ok(content_string)
    }
}
// Encrypt a buffer with the given key and iv using AES-256/CBC/Pkcs encryption.
fn aes256_cbc_encrypt(data: &[u8], key: &[u8], iv: &[u8])->Result<Vec<u8>,symmetriccipher::SymmetricCipherError>{
    let mut encryptor=aes::cbc_encryptor(
        aes::KeySize::KeySize256,
        key,
        iv,
        blockmodes::PkcsPadding);

    let mut final_result=Vec::<u8>::new();
    let mut read_buffer=buffer::RefReadBuffer::new(data);
    let mut buffer=[0;4096];
    let mut write_buffer=buffer::RefWriteBuffer::new(&mut buffer);

    loop{
        let result=(encryptor.encrypt(&mut read_buffer,&mut write_buffer,true))?;

        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));

        match result {
            BufferResult::BufferUnderflow=>break,
            BufferResult::BufferOverflow=>{},
        }
    }

    Ok(final_result)
}

/// Decrypts a buffer with the given key and iv using AES-256/CBC/Pkcs encryption.
fn aes256_cbc_decrypt(encrypted_data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
    let mut decryptor = aes::cbc_decryptor(
        aes::KeySize::KeySize256,
        key,
        iv,
        blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = (decryptor.decrypt(&mut read_buffer, &mut write_buffer, true))?;
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => { }
        }
    }

    Ok(final_result)
}
#[cfg(test)]
mod tests {
    use base64;
    use super::PrpCrypto;

    #[test]
    fn test_prpcrypto_encrypt() {
        let encoding_aes_key = "kWxPEV2UEDyxWpmPdKC3F4dgPDmOvfKX1HGnEUDS1aR=";
        let key = base64::decode(encoding_aes_key).unwrap();
        let prp = PrpCrypto::new(&key);
        let encrypted = prp.encrypt("test", "rust").unwrap();
        assert_eq!("9s4gMv99m88kKTh/H8IdkNiFGeG9pd7vNWl50fGRWXY=", &encrypted);
    }

    #[test]
    fn test_prpcrypto_decrypt() {
        let encoding_aes_key = "kWxPEV2UEDyxWpmPdKC3F4dgPDmOvfKX1HGnEUDS1aR=";
        let key = base64::decode(encoding_aes_key).unwrap();
        let prp = PrpCrypto::new(&key);
        let decrypted = prp.decrypt("9s4gMv99m88kKTh/H8IdkNiFGeG9pd7vNWl50fGRWXY=", "rust").unwrap();
        assert_eq!("test", &decrypted);
    }
}
