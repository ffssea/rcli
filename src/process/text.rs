use crate::TextSignFormat;
use anyhow::{Ok, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use std::{collections::HashMap, fs, io::Read, path::Path};

use super::process_genpass;

pub trait TextSigner {
    /// Signs the given reader and returns the signature
    // &dyn Read 性能较差，更加灵活
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerifier {
    // &dyn Read 性能好，代码量大
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool>;
    // fn verify<R: Read>(&self, reader: R, sig: &[u8]) -> Result<bool>;
    // fn verify(&self, reader: impl Read, sig: &[u8]) -> Result<bool>;
}

pub trait KeyGenerator {
    fn generate() -> Result<HashMap<&'static str, Vec<u8>>>;
}

pub struct Blake3 {
    key: [u8; 32],
}

pub struct Ed25519Signer {
    key: SigningKey,
}

pub struct Ed25519Verifier {
    key: VerifyingKey,
}

impl TextSigner for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let ret = blake3::keyed_hash(&self.key, &buf);
        Ok(ret.as_bytes().to_vec())
    }
}

impl TextVerifier for Blake3 {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let ret = blake3::keyed_hash(&self.key, &buf);
        Ok(ret.as_bytes() == sig)
    }
}

impl TextSigner for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = self.key.sign(&buf);
        Ok(sig.to_bytes().to_vec())
    }
}

impl TextVerifier for Ed25519Verifier {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = (&sig[..64]).try_into()?;
        let signature = Signature::from_bytes(sig);
        Ok(self.key.verify(&buf, &signature).is_ok())
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<HashMap<&'static str, Vec<u8>>> {
        let key = process_genpass(32, true, true, true, true)?;
        let key = key.as_bytes().to_vec();
        let mut map: HashMap<&'static str, Vec<u8>> = HashMap::new();
        map.insert("blake3.txt", key);
        Ok(map)
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> Result<HashMap<&'static str, Vec<u8>>> {
        let mut csprng = OsRng;
        let sk: SigningKey = SigningKey::generate(&mut csprng);
        let pk = sk.verifying_key().to_bytes().to_vec();
        let sk = sk.to_bytes().to_vec();
        let mut map: HashMap<&'static str, Vec<u8>> = HashMap::new();
        map.insert("ed25519.sk", sk);
        map.insert("ed25519.pk", pk);
        Ok(map)
    }
}

impl Blake3 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = &key[..32];
        let key = key.try_into()?;
        let signer = Blake3 { key };
        Ok(signer)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl Ed25519Signer {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = SigningKey::from_bytes(key);
        Self { key }
    }
    pub fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = (&key[..32]).try_into()?;
        Ok(Self::new(key))
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl Ed25519Verifier {
    pub fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = (&key[..32]).try_into()?;
        let key = VerifyingKey::from_bytes(key)?;
        Ok(Self { key })
    }
}

pub fn process_text_sign(
    reader: &mut dyn Read,
    key: &[u8],
    format: TextSignFormat,
) -> Result<Vec<u8>> {
    let signer: Box<dyn TextSigner> = match format {
        TextSignFormat::Blake3 => Box::new(Blake3::try_new(key)?),
        TextSignFormat::Ed25519 => Box::new(Ed25519Signer::try_new(key)?),
    };
    signer.sign(reader)
}

pub fn process_text_verify(
    reader: &mut dyn Read,
    key: &[u8],
    sig: &[u8],
    format: TextSignFormat,
) -> Result<bool> {
    let verifier: Box<dyn TextVerifier> = match format {
        TextSignFormat::Blake3 => Box::new(Blake3::try_new(key)?),
        TextSignFormat::Ed25519 => Box::new(Ed25519Verifier::try_new(key)?),
    };
    verifier.verify(reader, sig)
}

pub fn process_text_genkey(format: TextSignFormat) -> Result<HashMap<&'static str, Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

    const KEY: &[u8] = include_bytes!("../../fixtures/blake3.txt");

    #[test]
    fn test_process_text_sign() -> Result<()> {
        let mut reader = "hello".as_bytes();
        let mut reader1 = "hello".as_bytes();
        let format = TextSignFormat::Blake3;
        let sig = process_text_sign(&mut reader, KEY, format)?;
        let ret = process_text_verify(&mut reader1, KEY, &sig, format)?;
        assert!(ret);
        Ok(())
    }

    #[test]
    fn test_process_text_verify() -> Result<()> {
        let mut reader = "hello".as_bytes();
        let format = TextSignFormat::Blake3;
        let sig = "33Ypo4rveYpWmJKAiGnnse-wHQhMVujjmcVkV4Tl43k";
        let sig = URL_SAFE_NO_PAD.decode(sig)?;
        let ret = process_text_verify(&mut reader, KEY, &sig, format)?;
        assert!(ret);
        Ok(())
    }
}
