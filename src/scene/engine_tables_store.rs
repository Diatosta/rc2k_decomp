//! The engine's lookup tables and the object file's room, typed (phase 7): IDA 0xA23AE4 ..
//! 0xA8BB54, in the image's zeroed tail (`race::store::buffers`), made by
//! `scene::fill_engine_tables` at start: the sines (the cosines are the same table 0x400
//! places on), the arctangents, the noise bytes, the room the object files are read into
//! and the shuffled words. Names are readings unless IDA's.

use crate::session::{Bytes, Float, Word};

#[repr(C)]
pub struct EngineTables {
    /// +0x0 (0xA23AE4): `g_sines`: 0x1400 floats, the sine of n times pi over 0x800; `g_cosines` (0xA24AE4) is this table from entry 0x400 (`math::turn_scratch_xz`, `scene::matrix`, `marks::`, `hud::draw_disc`).
    pub sines: [Float; 0x1400],
    /// +0x5000 (0xA28AE4): `g_arctangents`: 0x1000 words, the angle of a tangent (`scene::fill_engine_tables`).
    pub arctangents: [Word; 0x1000],
    /// +0x7000 (0xA2AAE4): `g_noiseBytes`: 0x1070 bytes of noise; the noise maps `unk_A2AB04`, `unk_A2AB34` are in it, at +0x20 and +0x50 (`physics::probe_ground`, `physics::surface_noise`, `race::frame_draw`).
    pub noise_bytes: Bytes<0x1070>,
    /// +0x8070 (0xA2BB54): `g_objectFile`: the room the object file is read into, 0x40000 bytes (+4 points, +8 faces, +0xC normals, +0x2C the points: `scene::objects`, `scene::load_object`, `scene::add_object_shape`).
    pub object_file: Bytes<0x40000>,
    /// +0x48070 (0xA6BB54): `g_shuffledWords`: the numbers 0 .. 0xFFFF shuffled (`race::muddy_texel`).
    pub shuffled_words: [Word; 0x10000],
}

pub struct InBuffers;

impl core::ops::Deref for InBuffers {
    type Target = EngineTables;
    fn deref(&self) -> &EngineTables {
        // SAFETY: the block lives as long as the program and the piece is aligned to 4 (the
        // test checks); every field is an atomic or a cell.
        unsafe { &*(piece_addr() as *const EngineTables) }
    }
}

pub static ENGINE_TABLES: InBuffers = InBuffers;

/// The sine table's cell `n`: the callers index with an angle's top bits (below 0x1000), and
/// the cosine is the cell 0x400 on.
pub fn sine(n: usize) -> &'static Float {
    // SAFETY: the piece is one block of cells; the address is the original's own.
    unsafe { &*(ENGINE_TABLES.sines[0].addr().wrapping_add(4 * n) as *const Float) }
}

/// The cosine of table place `n` (`g_cosines[n]`, the sine table's cell `n + 0x400`).
pub fn cosine(n: usize) -> &'static Float {
    sine(n.wrapping_add(0x400))
}

/// IDA 0xA23AE4, the first byte of the piece.
pub const START: usize = 0xA23AE4;
/// IDA 0xA8BB54, the end of the piece.
pub const END: usize = 0xA8BB54;

const _: () = assert!(END - START == core::mem::size_of::<EngineTables>());

/// Where IDA 0xA23AE4 is now.
pub fn piece_addr() -> usize {
    crate::moved::place(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_buffers() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
        );
        assert_eq!(offset_of!(EngineTables, arctangents), 0xA28AE4 - START);
        assert_eq!(offset_of!(EngineTables, noise_bytes), 0xA2AAE4 - START);
        assert_eq!(offset_of!(EngineTables, object_file), 0xA2BB54 - START);
        assert_eq!(offset_of!(EngineTables, shuffled_words), 0xA6BB54 - START);
        assert_eq!(
            ENGINE_TABLES.sines[0x400].addr(),
            crate::moved::place(0xA24AE4)
        );
    }
}
