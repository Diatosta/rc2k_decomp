//! The language's set-up file, `files\<locale>\*.loc` (read by `strings::load_language`).

use crate::Invalid;

/// What the game uses of the 0x30-byte `.loc` file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Locale {
    /// Byte +8: the kind of notice the version shows (2 offers registration, 7 is the
    /// Japanese version).
    pub notice_kind: u8,
    /// Byte +0xF: the units (0 imperial, 1 metric).
    pub units: u8,
}

impl Locale {
    /// The file's bytes; it must start "LOCL".
    pub fn parse(file: &[u8; 0x30]) -> Result<Self, Invalid> {
        if &file[..4] != b"LOCL" {
            return Err(Invalid("a .loc file that does not start LOCL"));
        }
        Ok(Locale {
            notice_kind: file[8],
            units: file[0xF],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        let mut file = [0u8; 0x30];
        assert!(Locale::parse(&file).is_err());
        file[..4].copy_from_slice(b"LOCL");
        file[8] = 2;
        file[0xF] = 1;
        assert_eq!(
            Locale::parse(&file).unwrap(),
            Locale {
                notice_kind: 2,
                units: 1
            }
        );
    }
}
