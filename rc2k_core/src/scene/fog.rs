//! The fog's alpha at a depth (converted 2026-10-21 from `scene::fog_at` 0x44E260): the
//! depth (0 when below 0) over 0x2000, held below the table's length (the last entry
//! past it); that entry of the fog table in the top byte, flipped.

use crate::Invalid;

/// The fog table's bytes.
pub const TABLE_LEN: i32 = 0x1000;

/// `fog_at`: the alpha (in the top byte) at `depth` with a table of `len` entries
/// (`entry(n)` asked for the one used). Refuses a length of 0 or less or past the table
/// (the original reads past it).
pub fn fog_at(depth: u32, len: i32, entry: impl FnOnce(usize) -> u8) -> Result<u32, Invalid> {
    if len <= 0 || len > TABLE_LEN {
        return Err(Invalid("a fog table length of 0 or past the table"));
    }
    let d = if (depth as i32) < 0 { 0 } else { depth };
    let mut i = d >> 13;
    if (i as i32).wrapping_sub(len) >= 0 {
        i = (len - 1) as u32;
    }
    Ok((u32::from(entry(i as usize)) << 24) ^ 0xFF00_0000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alphas() {
        let table = |n: usize| n as u8;
        assert_eq!(fog_at(0x4000, 0x10, table), Ok(0xFD00_0000));
        assert_eq!(fog_at(u32::MAX, 0x10, table), Ok(0xFF00_0000));
        assert_eq!(fog_at(0x7FFF_FFFF, 0x10, table), Ok(0xF000_0000));
        assert!(fog_at(0, 0, table).is_err());
        assert!(fog_at(0, 0x1001, table).is_err());
    }
}
