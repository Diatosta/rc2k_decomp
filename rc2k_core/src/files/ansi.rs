//! The game's texts: bytes in the ANSI code page (Windows-1252) and the text they stand for.

/// Windows-1252's characters for 0x80 .. 0x9F (the rest of the code page is Latin-1).
const CP1252_HIGH: [char; 32] = [
    '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8D}', 'Ž', '\u{8F}',
    '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9D}', 'ž', 'Ÿ',
];

/// The game's bytes as text, read the way the Win32 `A` functions read them (the ANSI code
/// page, Windows-1252).
pub fn ansi_text(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| match b {
            0x80..=0x9F => CP1252_HIGH[(b - 0x80) as usize],
            _ => b as char,
        })
        .collect()
}

/// Text as the game's bytes (Windows-1252, `ansi_text`'s inverse); a character the code
/// page lacks becomes `?`, as Windows' conversion writes it.
pub fn ansi_bytes(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| match c as u32 {
            0..=0x7F | 0xA0..=0xFF => c as u8,
            _ => CP1252_HIGH
                .iter()
                .position(|&h| h == c)
                .map_or(b'?', |n| 0x80 + n as u8),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_text_is_windows_1252() {
        assert_eq!(ansi_text(b"files\\b\\tx1822.pcx"), "files\\b\\tx1822.pcx");
        assert_eq!(ansi_text(&[0x80, 0xE7, 0xE3, 0x9C]), "€çãœ");
        assert_eq!(ansi_bytes("€çãœ"), [0x80, 0xE7, 0xE3, 0x9C]);
        assert_eq!(ansi_bytes("a\u{3B1}"), b"a?");
    }
}
