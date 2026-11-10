//! `gfx::components_to_4444_rows` (0x40750D): the plain picture converter for pixel format
//! number 4, two bytes a pixel with four bits for each component.
//!
//! A pixel's first byte is the top four bits of its first component in the low half and
//! of its second in the high half; its second byte the same of the third and fourth.
//! Reached only through the converter table `dword_4F5288`, where `gfx::install_converters`
//! puts it when the processor has no MMX; with MMX 0x407276 stands there instead (not
//! ported). So on a machine of today it is never called.
//! Refuses a row or a row count of 0 (the original counts through all 32 bits). The name
//! is a reading. The Rust version works in place on the picture's bytes,
//! its only call (from = to, one row, ebp 0).

use crate::errors::BadInput;

pub const COMPONENTS_TO_4444_ROWS: u32 = 0x40750D;

/// In place on the picture's bytes (`gfx::convert_picture`'s call: one row of `pixels`, from
/// and to the buffer's start; each pixel's two bytes written after its four are read).
/// Refuses no pixels and a buffer shorter than them.
pub fn components_to_4444_in_place(bytes: &mut [u8], pixels: usize) -> Result<(), BadInput> {
    if pixels == 0 {
        return Err(BadInput("no pixels to convert"));
    }
    if bytes.len() < pixels * 4 {
        return Err(BadInput("a picture shorter than its pixels"));
    }
    for n in 0..pixels {
        // Converted: `rc2k_core::gfx::pixels::to_4444`.
        let components: [u8; 4] = bytes[4 * n..4 * n + 4].try_into().expect("four bytes");
        let [low, high] = rc2k_core::gfx::pixels::to_4444(components);
        bytes[2 * n] = low;
        bytes[2 * n + 1] = high;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_are_packed_in_place() {
        let mut bytes = vec![0x12, 0x34, 0x56, 0x78, 0xF0, 0xE0, 0xD0, 0xC0];
        components_to_4444_in_place(&mut bytes, 2).unwrap();
        let first = rc2k_core::gfx::pixels::to_4444([0x12, 0x34, 0x56, 0x78]);
        let second = rc2k_core::gfx::pixels::to_4444([0xF0, 0xE0, 0xD0, 0xC0]);
        assert_eq!(bytes[..4], [first[0], first[1], second[0], second[1]]);
        assert!(components_to_4444_in_place(&mut bytes, 3).is_err());
    }
}
