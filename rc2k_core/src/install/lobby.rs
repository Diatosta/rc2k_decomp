//! The game registered as a DirectPlay lobby program (converted 2026-10-21 from
//! `install::register_lobby` 0x419030 and its GUID text, `sub_40171A`).

use crate::strings::format::hex;

/// A GUID as the registry's `Guid` value holds it: `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}`,
/// upper case: the first dword and the two words as numbers (lowest byte first in the
/// GUID), then the eight last bytes in order.
pub fn guid_text(guid: &[u8; 16]) -> Vec<u8> {
    let word = |at: usize| u16::from_le_bytes([guid[at], guid[at + 1]]) as u32;
    let digits = |value: u32, n: u32| hex(value, n).unwrap_or_default();
    let mut out = vec![b'{'];
    out.extend(digits(
        u32::from_le_bytes([guid[0], guid[1], guid[2], guid[3]]),
        8,
    ));
    out.push(b'-');
    out.extend(digits(word(4), 4));
    out.push(b'-');
    out.extend(digits(word(6), 4));
    out.push(b'-');
    out.extend(digits((guid[8] as u32) << 8 | guid[9] as u32, 4));
    out.push(b'-');
    for &b in &guid[10..] {
        out.extend(digits(b as u32, 2));
    }
    out.push(b'}');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guid() {
        let g = [
            0x78, 0x56, 0x34, 0x12, 0x34, 0x12, 0xCD, 0xAB, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF,
        ];
        assert_eq!(
            guid_text(&g),
            b"{12345678-1234-ABCD-0123-456789ABCDEF}".to_vec()
        );
    }
}
