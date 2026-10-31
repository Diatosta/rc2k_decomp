//! The surface's texture places and the emitters' lives and colours (IDA 0x935766 .. 0x939DB2), out of the marks' block (phase 7, typed state).

use crate::session::{Bytes, Dword};

/// The surface's texture places and the emitters' lives and colours (IDA 0x935766 .. 0x939DB2, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SurfaceTables {
    /// 0x935766: `g_marksCellPlaces` (0x40 cells of four floats, 16 bytes each) then `g_marksSubcellPlaces` at 0x935B66 (0x40 x 0x100 bytes: 4 x 4 sub-cells of 16 bytes per cell); the original reads one cell and 0x10 sub-cells past them (see `SurfaceTables::word`).
    pub places: Bytes<17408>,
    /// 0x939B66: Not named: 16 bytes (a gap, or unreferenced data).
    pub _gap_939b66: Bytes<16>,
    /// 0x939B76: `g_marksParticleLives`: the life by a wheel's flags' bits 6, 7 (`effects::emit_particle_6`).
    pub lives_by_flags: [Dword; 4],
    /// 0x939B86: `dword_939B86`: the life by the level (`effects::emit_twin`; 9 entries are allowed, the last is `dword_939BA6`, the tint).
    pub lives_by_level: [Dword; 9],
    /// 0x939BAA: `g_marksWheelFlags2`: the wheel's +0x24 kept.
    pub wheel_flags2: Dword,
    /// 0x939BAE: `g_marksWheelFlags`: the wheel's +0x20 kept.
    pub wheel_flags: Dword,
    /// 0x939BB2: `dword_939BB2`: the 128 colours a streak takes by its age (the other table, 257 of them, follows at 0x939DB2).
    pub colours: [Dword; 128],
}

pub static SURFACE: SurfaceTables = SurfaceTables {
    places: Bytes::new([0; 17408]),
    _gap_939b66: Bytes::new([
        0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]),
    lives_by_flags: [
        Dword::new(0x0),
        Dword::new(0xFA),
        Dword::new(0x1F4),
        Dword::new(0x3E8),
    ],
    lives_by_level: [
        Dword::new(0x7D),
        Dword::new(0xFA),
        Dword::new(0x177),
        Dword::new(0x1F4),
        Dword::new(0x28F),
        Dword::new(0x2EE),
        Dword::new(0x36B),
        Dword::new(0x3E8),
        Dword::new(0x0),
    ],
    wheel_flags2: Dword::new(0x0),
    wheel_flags: Dword::new(0x0),
    colours: [
        Dword::new(0xFF0000),
        Dword::new(0xFF0200),
        Dword::new(0xFF0400),
        Dword::new(0xFF0600),
        Dword::new(0xFF0800),
        Dword::new(0xFF0A00),
        Dword::new(0xFF0C00),
        Dword::new(0xFF0E00),
        Dword::new(0xFF1000),
        Dword::new(0xFF1200),
        Dword::new(0xFF1400),
        Dword::new(0xFF1600),
        Dword::new(0xFF1800),
        Dword::new(0xFF1A00),
        Dword::new(0xFF1C00),
        Dword::new(0xFF1E00),
        Dword::new(0xFF2000),
        Dword::new(0xFF2200),
        Dword::new(0xFF2400),
        Dword::new(0xFF2600),
        Dword::new(0xFF2800),
        Dword::new(0xFF2A00),
        Dword::new(0xFF2C00),
        Dword::new(0xFF2E00),
        Dword::new(0xFF3000),
        Dword::new(0xFF3200),
        Dword::new(0xFF3400),
        Dword::new(0xFF3600),
        Dword::new(0xFF3800),
        Dword::new(0xFF3A00),
        Dword::new(0xFF3C00),
        Dword::new(0xFF3E00),
        Dword::new(0xFF4000),
        Dword::new(0xFF4200),
        Dword::new(0xFF4400),
        Dword::new(0xFF4600),
        Dword::new(0xFF4800),
        Dword::new(0xFF4A00),
        Dword::new(0xFF4C00),
        Dword::new(0xFF4E00),
        Dword::new(0xFF5000),
        Dword::new(0xFF5200),
        Dword::new(0xFF5400),
        Dword::new(0xFF5600),
        Dword::new(0xFF5800),
        Dword::new(0xFF5A00),
        Dword::new(0xFF5C00),
        Dword::new(0xFF5E00),
        Dword::new(0xFF6000),
        Dword::new(0xFF6200),
        Dword::new(0xFF6400),
        Dword::new(0xFF6600),
        Dword::new(0xFF6800),
        Dword::new(0xFF6A00),
        Dword::new(0xFF6C00),
        Dword::new(0xFF6E00),
        Dword::new(0xFF7000),
        Dword::new(0xFF7200),
        Dword::new(0xFF7400),
        Dword::new(0xFF7600),
        Dword::new(0xFF7800),
        Dword::new(0xFF7A00),
        Dword::new(0xFF7C00),
        Dword::new(0xFF7E00),
        Dword::new(0xFF8000),
        Dword::new(0xFF8200),
        Dword::new(0xFF8400),
        Dword::new(0xFF8600),
        Dword::new(0xFF8800),
        Dword::new(0xFF8A00),
        Dword::new(0xFF8C00),
        Dword::new(0xFF8E00),
        Dword::new(0xFF9000),
        Dword::new(0xFF9200),
        Dword::new(0xFF9400),
        Dword::new(0xFF9600),
        Dword::new(0xFF9800),
        Dword::new(0xFF9A00),
        Dword::new(0xFF9C00),
        Dword::new(0xFF9E00),
        Dword::new(0xFFA000),
        Dword::new(0xFFA200),
        Dword::new(0xFFA400),
        Dword::new(0xFFA600),
        Dword::new(0xFFA800),
        Dword::new(0xFFAA00),
        Dword::new(0xFFAC00),
        Dword::new(0xFFAE00),
        Dword::new(0xFFB000),
        Dword::new(0xFFB200),
        Dword::new(0xFFB400),
        Dword::new(0xFFB600),
        Dword::new(0xFFB800),
        Dword::new(0xFFBA00),
        Dword::new(0xFFBC00),
        Dword::new(0xFFBE00),
        Dword::new(0xFFC000),
        Dword::new(0xFFC200),
        Dword::new(0xFFC400),
        Dword::new(0xFFC600),
        Dword::new(0xFFC800),
        Dword::new(0xFFCA00),
        Dword::new(0xFFCC00),
        Dword::new(0xFFCE00),
        Dword::new(0xFFD000),
        Dword::new(0xFFD200),
        Dword::new(0xFFD400),
        Dword::new(0xFFD600),
        Dword::new(0xFFD800),
        Dword::new(0xFFDA00),
        Dword::new(0xFFDC00),
        Dword::new(0xFFDE00),
        Dword::new(0xFFE000),
        Dword::new(0xFFE200),
        Dword::new(0xFFE400),
        Dword::new(0xFFE600),
        Dword::new(0xFFE800),
        Dword::new(0xFFEA00),
        Dword::new(0xFFEC00),
        Dword::new(0xFFEE00),
        Dword::new(0xFFF000),
        Dword::new(0xFFF200),
        Dword::new(0xFFF400),
        Dword::new(0xFFF600),
        Dword::new(0xFFF800),
        Dword::new(0xFFFA00),
        Dword::new(0xFFFC00),
        Dword::new(0xFFFE00),
    ],
};

impl SurfaceTables {
    pub const START: usize = 0x935766;
    pub const LEN: usize = 0x464C;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&SURFACE as *const SurfaceTables) as usize
    }
}

impl SurfaceTables {
    /// The dword at byte `at` of the places (any alignment). The original reads one cell
    /// (`effects::draw_cell_sprite`: cell 0x40) and 0x10 sub-cells (`effects::draw_subcell_sprite`:
    /// sub-cells 0x400 .. 0x40F) past their tables, into the bytes that follow: the read
    /// goes on into the struct's next fields, as the image's neighbours were read.
    pub fn word(&self, at: usize) -> u32 {
        assert!(at + 4 <= Self::LEN);
        crate::pointers::read_or_end(
            crate::pointers::game_address(self as *const Self) + at as u32,
            "a surface word",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn the_surface_tables_starts_as_the_games() {
        assert_eq!(size_of::<SurfaceTables>(), SurfaceTables::LEN);
        let p = &SURFACE as *const SurfaceTables as *const u8;
        let bytes: Vec<u8> = (0..size_of::<SurfaceTables>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            bytes == crate::moved::image_bytes(SurfaceTables::START, SurfaceTables::LEN),
            "SURFACE starts differently from the image"
        );
        assert_eq!(
            offset_of!(SurfaceTables, places),
            0x935766 - SurfaceTables::START
        );
        assert_eq!(
            offset_of!(SurfaceTables, lives_by_flags),
            0x939B76 - SurfaceTables::START
        );
        assert_eq!(
            offset_of!(SurfaceTables, lives_by_level),
            0x939B86 - SurfaceTables::START
        );
        assert_eq!(
            offset_of!(SurfaceTables, wheel_flags2),
            0x939BAA - SurfaceTables::START
        );
        assert_eq!(
            offset_of!(SurfaceTables, wheel_flags),
            0x939BAE - SurfaceTables::START
        );
        assert_eq!(
            offset_of!(SurfaceTables, colours),
            0x939BB2 - SurfaceTables::START
        );
    }
}
