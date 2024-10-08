use rand::prelude::*;
use zxcvbn::zxcvbn;

const UPPER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijklmnpqrstuvwxyz";
const NUMBER: &[u8] = b"123456789";
const SYMBOL: &[u8] = b"!@#$%^&*_+-=";

pub fn process_genpass(
    length: u8,
    upper: bool,
    lower: bool,
    number: bool,
    symbol: bool,
) -> anyhow::Result<String> {
    let mut rng = rand::thread_rng();
    let mut password = Vec::new();
    let mut chars = Vec::new();
    if upper {
        chars.extend_from_slice(UPPER);
        password.push(UPPER[rng.gen_range(0..UPPER.len())]);
        // password.push(*UPPER.choose(&mut rng).expect("UPPER won't be empty"));
    }
    if lower {
        chars.extend_from_slice(LOWER);
        password.push(LOWER[rng.gen_range(0..LOWER.len())]);
    }
    if number {
        chars.extend_from_slice(NUMBER);
        password.push(NUMBER[rng.gen_range(0..NUMBER.len())]);
    }
    if symbol {
        chars.extend_from_slice(SYMBOL);
        password.push(SYMBOL[rng.gen_range(0..SYMBOL.len())]);
    }
    for _ in 0..(length - password.len() as u8) {
        password.push(chars[rng.gen_range(0..chars.len())]);
    }
    password.shuffle(&mut rng);
    let passwd = String::from_utf8(password)?;
    let estimate = zxcvbn(&passwd, &[]);
    eprintln!("password strength: {}", estimate.score());
    Ok(passwd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_genpass() {
        let passwd = process_genpass(128, true, true, true, true).unwrap();
        println!("{}", passwd);
    }
}
