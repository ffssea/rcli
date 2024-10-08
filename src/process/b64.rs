use crate::{cli::Base64Format, get_reader};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine as _,
};

pub fn process_encode(input: &str, format: Base64Format) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    println!("{}", buf);
    let encoded = match format {
        Base64Format::UrlSafe => URL_SAFE_NO_PAD.encode(&buf),
        Base64Format::Standard => STANDARD.encode(&buf),
    };
    // println!("{}", encoded);
    Ok(encoded)
}

pub fn process_decode(input: &str, format: Base64Format) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let buf = buf.trim();
    let decoded = match format {
        Base64Format::UrlSafe => URL_SAFE_NO_PAD.decode(buf)?,
        Base64Format::Standard => STANDARD.decode(buf)?,
    };
    // println!("{}", String::from_utf8(decoded.clone())?);
    Ok(String::from_utf8(decoded)?)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    #[test]
    fn test_process_encode() {
        let input = "Cargo.toml";
        let ret = process_encode(input, Base64Format::Standard).unwrap();
        fs::write("fixtures/cargo_toml_b64.txt", ret).unwrap();

        assert!(process_encode(input, Base64Format::Standard).is_ok())
    }

    #[test]
    fn test_process_decode() {
        let input = "fixtures/cargo_toml_b64.txt";
        assert!(process_decode(input, Base64Format::Standard).is_ok())
    }
}
