//! Base 45 (RFC 9285): the text form of the code. Its letters are exactly the characters a QR
//! code's alphanumeric mode takes, so the same text fills a QR code with 5.5 bits a character
//! (a byte-mode code of base 64 text would hold a quarter less).

use super::CodeError;

const ALPHABET: &[u8; 45] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

/// `bytes` as text.
pub fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() / 2 * 3 + 2);
    for pair in bytes.chunks(2) {
        let mut v = pair.iter().fold(0u32, |v, &b| v * 256 + u32::from(b));
        let digits = if pair.len() == 2 { 3 } else { 2 };
        for _ in 0..digits {
            out.push(char::from(ALPHABET[(v % 45) as usize]));
            v /= 45;
        }
    }
    out
}

fn digit(c: u8) -> Result<u32, CodeError> {
    ALPHABET
        .iter()
        .position(|&a| a == c)
        .map(|p| p as u32)
        .ok_or(CodeError::NotACode)
}

/// The bytes `text` holds. Refused: a letter outside the alphabet, a length no encoding gives,
/// a group too large for its bytes.
pub fn decode(text: &str) -> Result<Vec<u8>, CodeError> {
    let text = text.as_bytes();
    if text.len() % 3 == 1 {
        return Err(CodeError::NotACode);
    }
    let mut out = Vec::with_capacity(text.len() / 3 * 2 + 1);
    for group in text.chunks(3) {
        let mut v = 0u32;
        for &c in group.iter().rev() {
            v = v * 45 + digit(c)?;
        }
        if group.len() == 3 {
            let v = u16::try_from(v).map_err(|_| CodeError::NotACode)?;
            out.extend_from_slice(&v.to_be_bytes());
        } else {
            out.push(u8::try_from(v).map_err(|_| CodeError::NotACode)?);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rfcs_examples() {
        assert_eq!(encode(b"AB"), "BB8");
        assert_eq!(encode(b"Hello!!"), "%69 VD92EX0");
        assert_eq!(encode(b"base-45"), "UJCLQE7W581");
        assert_eq!(decode("QED8WEX0").unwrap(), b"ietf!");
    }

    #[test]
    fn round_trips_every_length() {
        for n in 0..40u8 {
            let bytes: Vec<u8> = (0..n)
                .map(|i| i.wrapping_mul(97).wrapping_add(250))
                .collect();
            assert_eq!(decode(&encode(&bytes)).unwrap(), bytes);
        }
    }

    #[test]
    fn bad_text_is_refused() {
        assert!(decode("a").is_err());
        assert!(decode("GGW").is_err());
        assert!(decode("::").is_err());
        assert!(decode("ABCD").is_err());
    }
}
