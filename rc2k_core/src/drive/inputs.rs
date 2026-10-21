//! The race's input words (converted 2026-10-21 from `drive::pack_inputs` 0x409328,
//! `unpack_inputs` 0x409338 and `any_key_down` 0x444F55): four words packed into one for
//! the replays, and whether any key is down.

/// `drive::pack_inputs`: `a | ((b << 9 | c) << 9)`, << 4 (the original's `shl`s, high
/// bits lost).
pub fn pack_inputs(a: u32, b: u32, c: u32) -> u32 {
    (a | ((b << 9 | c) << 9)) << 4
}

/// `drive::unpack_inputs`: a packed word taken apart: [bits 4 .. 12, the top ten (signed,
/// `sar` 22), bits 13 .. 21] (the original's eax, ecx, edx).
pub fn unpack_inputs(x: u32) -> [u32; 3] {
    [
        (x >> 4) & 0x1FF,
        ((x as i32) >> 22) as u32,
        (x >> 13) & 0x1FF,
    ]
}

/// The key bytes (`g_keysPressed`).
pub const KEY_COUNT: usize = 0x80;

/// `drive::any_key_down`: the key bytes or'ed together (a key is down when not 0).
pub fn keys_or(keys: &[u8; KEY_COUNT]) -> u8 {
    keys.iter().fold(0, |o, &k| o | k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed() {
        let w = pack_inputs(0x1AB, 0x155, 0x0CD);
        assert_eq!(w, (0x1AB | (0x155 << 18) | (0xCD << 9)) << 4);
        assert_eq!(unpack_inputs(w), [0x1AB, 0x155, 0xCD]);
        assert_eq!(unpack_inputs(0x8000_0000)[1], 0xFFFF_FE00);
    }

    #[test]
    fn keys() {
        let mut k = [0u8; KEY_COUNT];
        assert_eq!(keys_or(&k), 0);
        k[3] = 0x80;
        k[0x7F] = 1;
        assert_eq!(keys_or(&k), 0x81);
    }
}
