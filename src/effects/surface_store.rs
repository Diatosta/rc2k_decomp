//! The surface's texture places and the emitters' lives and colours (IDA 0x935766 .. 0x939DB2).

use crate::session::{PlainBytes, PlainDword};

/// The surface's texture places and the emitters' lives and colours (IDA 0x935766 .. 0x939DB2).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SurfaceTables {
    /// 0x935766: `g_marksCellPlaces` (0x40 cells of four floats, 16 bytes each) then `g_marksSubcellPlaces` at 0x935B66 (0x40 x 0x100 bytes: 4 x 4 sub-cells of 16 bytes per cell); the original reads one cell and 0x10 sub-cells past them (see `SurfaceTables::word`).
    pub places: PlainBytes<17408>,
    /// 0x939B66: Not named: 16 bytes (a gap, or unreferenced data).
    pub _gap_939b66: PlainBytes<16>,
    /// 0x939B76: `g_marksParticleLives`: the life by a wheel's flags' bits 6, 7 (`effects::emit_wheel_fade_particle`).
    pub lives_by_flags: [PlainDword; 4],
    /// 0x939B86: `dword_939B86`: the life by the level (`effects::emit_twin`; 9 entries are allowed, the last is `dword_939BA6`, the tint).
    pub lives_by_level: [PlainDword; 9],
    /// 0x939BAA: `g_marksWheelFlags2`: the wheel's +0x24 kept.
    pub wheel_flags2: PlainDword,
    /// 0x939BAE: `g_marksWheelFlags`: the wheel's +0x20 kept.
    pub wheel_flags: PlainDword,
    /// 0x939BB2: `dword_939BB2`: the 128 colours a streak takes by its age (the other table, 257 of them, follows at 0x939DB2).
    pub colours: [PlainDword; 128],
}

impl SurfaceTables {
    pub const fn new() -> Self {
        SurfaceTables {
            places: PlainBytes::new([0; 17408]),
            _gap_939b66: PlainBytes::new([
                0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00,
            ]),
            lives_by_flags: [
                PlainDword::new(0x0),
                PlainDword::new(0xFA),
                PlainDword::new(0x1F4),
                PlainDword::new(0x3E8),
            ],
            lives_by_level: [
                PlainDword::new(0x7D),
                PlainDword::new(0xFA),
                PlainDword::new(0x177),
                PlainDword::new(0x1F4),
                PlainDword::new(0x28F),
                PlainDword::new(0x2EE),
                PlainDword::new(0x36B),
                PlainDword::new(0x3E8),
                PlainDword::new(0x0),
            ],
            wheel_flags2: PlainDword::new(0x0),
            wheel_flags: PlainDword::new(0x0),
            colours: [
                PlainDword::new(0xFF0000),
                PlainDword::new(0xFF0200),
                PlainDword::new(0xFF0400),
                PlainDword::new(0xFF0600),
                PlainDword::new(0xFF0800),
                PlainDword::new(0xFF0A00),
                PlainDword::new(0xFF0C00),
                PlainDword::new(0xFF0E00),
                PlainDword::new(0xFF1000),
                PlainDword::new(0xFF1200),
                PlainDword::new(0xFF1400),
                PlainDword::new(0xFF1600),
                PlainDword::new(0xFF1800),
                PlainDword::new(0xFF1A00),
                PlainDword::new(0xFF1C00),
                PlainDword::new(0xFF1E00),
                PlainDword::new(0xFF2000),
                PlainDword::new(0xFF2200),
                PlainDword::new(0xFF2400),
                PlainDword::new(0xFF2600),
                PlainDword::new(0xFF2800),
                PlainDword::new(0xFF2A00),
                PlainDword::new(0xFF2C00),
                PlainDword::new(0xFF2E00),
                PlainDword::new(0xFF3000),
                PlainDword::new(0xFF3200),
                PlainDword::new(0xFF3400),
                PlainDword::new(0xFF3600),
                PlainDword::new(0xFF3800),
                PlainDword::new(0xFF3A00),
                PlainDword::new(0xFF3C00),
                PlainDword::new(0xFF3E00),
                PlainDword::new(0xFF4000),
                PlainDword::new(0xFF4200),
                PlainDword::new(0xFF4400),
                PlainDword::new(0xFF4600),
                PlainDword::new(0xFF4800),
                PlainDword::new(0xFF4A00),
                PlainDword::new(0xFF4C00),
                PlainDword::new(0xFF4E00),
                PlainDword::new(0xFF5000),
                PlainDword::new(0xFF5200),
                PlainDword::new(0xFF5400),
                PlainDword::new(0xFF5600),
                PlainDword::new(0xFF5800),
                PlainDword::new(0xFF5A00),
                PlainDword::new(0xFF5C00),
                PlainDword::new(0xFF5E00),
                PlainDword::new(0xFF6000),
                PlainDword::new(0xFF6200),
                PlainDword::new(0xFF6400),
                PlainDword::new(0xFF6600),
                PlainDword::new(0xFF6800),
                PlainDword::new(0xFF6A00),
                PlainDword::new(0xFF6C00),
                PlainDword::new(0xFF6E00),
                PlainDword::new(0xFF7000),
                PlainDword::new(0xFF7200),
                PlainDword::new(0xFF7400),
                PlainDword::new(0xFF7600),
                PlainDword::new(0xFF7800),
                PlainDword::new(0xFF7A00),
                PlainDword::new(0xFF7C00),
                PlainDword::new(0xFF7E00),
                PlainDword::new(0xFF8000),
                PlainDword::new(0xFF8200),
                PlainDword::new(0xFF8400),
                PlainDword::new(0xFF8600),
                PlainDword::new(0xFF8800),
                PlainDword::new(0xFF8A00),
                PlainDword::new(0xFF8C00),
                PlainDword::new(0xFF8E00),
                PlainDword::new(0xFF9000),
                PlainDword::new(0xFF9200),
                PlainDword::new(0xFF9400),
                PlainDword::new(0xFF9600),
                PlainDword::new(0xFF9800),
                PlainDword::new(0xFF9A00),
                PlainDword::new(0xFF9C00),
                PlainDword::new(0xFF9E00),
                PlainDword::new(0xFFA000),
                PlainDword::new(0xFFA200),
                PlainDword::new(0xFFA400),
                PlainDword::new(0xFFA600),
                PlainDword::new(0xFFA800),
                PlainDword::new(0xFFAA00),
                PlainDword::new(0xFFAC00),
                PlainDword::new(0xFFAE00),
                PlainDword::new(0xFFB000),
                PlainDword::new(0xFFB200),
                PlainDword::new(0xFFB400),
                PlainDword::new(0xFFB600),
                PlainDword::new(0xFFB800),
                PlainDword::new(0xFFBA00),
                PlainDword::new(0xFFBC00),
                PlainDword::new(0xFFBE00),
                PlainDword::new(0xFFC000),
                PlainDword::new(0xFFC200),
                PlainDword::new(0xFFC400),
                PlainDword::new(0xFFC600),
                PlainDword::new(0xFFC800),
                PlainDword::new(0xFFCA00),
                PlainDword::new(0xFFCC00),
                PlainDword::new(0xFFCE00),
                PlainDword::new(0xFFD000),
                PlainDword::new(0xFFD200),
                PlainDword::new(0xFFD400),
                PlainDword::new(0xFFD600),
                PlainDword::new(0xFFD800),
                PlainDword::new(0xFFDA00),
                PlainDword::new(0xFFDC00),
                PlainDword::new(0xFFDE00),
                PlainDword::new(0xFFE000),
                PlainDword::new(0xFFE200),
                PlainDword::new(0xFFE400),
                PlainDword::new(0xFFE600),
                PlainDword::new(0xFFE800),
                PlainDword::new(0xFFEA00),
                PlainDword::new(0xFFEC00),
                PlainDword::new(0xFFEE00),
                PlainDword::new(0xFFF000),
                PlainDword::new(0xFFF200),
                PlainDword::new(0xFFF400),
                PlainDword::new(0xFFF600),
                PlainDword::new(0xFFF800),
                PlainDword::new(0xFFFA00),
                PlainDword::new(0xFFFC00),
                PlainDword::new(0xFFFE00),
            ],
        }
    }
}

impl SurfaceTables {
    pub const START: usize = 0x935766;
    pub const LEN: usize = 0x464C;
}

impl SurfaceTables {
    /// The dword at byte `at` of the places (any alignment). The original reads one cell
    /// (`effects::draw_cell_sprite`: cell 0x40) and 0x10 sub-cells (`effects::draw_subcell_sprite`:
    /// sub-cells 0x400 .. 0x40F) past their tables, into the bytes that follow: the read
    /// goes on into the struct's next fields, as the image's neighbours were read.
    pub fn word(&self, at: usize) -> u32 {
        assert!(at + 4 <= Self::LEN);
        let mut word = [0u8; 4];
        crate::pointers::load_cells(self, at, &mut word)
            .unwrap_or_else(|e| crate::fatal::raise::end("a surface word", e));
        u32::from_le_bytes(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn the_surface_tables_starts_as_the_games() {
        assert_eq!(size_of::<SurfaceTables>(), SurfaceTables::LEN);
        let p = (&raw const crate::effects::owner::effects().masks.surface).cast::<u8>();
        let bytes: Vec<u8> = (0..size_of::<SurfaceTables>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            crate::test_support::first_values::holds(
                "effects.owner.EFFECTS.masks.surface/17996",
                &bytes
            ),
            "SURFACE starts differently from the image"
        );
        assert_eq!(offset_of!(SurfaceTables, places), 0x0);
        assert_eq!(offset_of!(SurfaceTables, lives_by_flags), 0x4410);
        assert_eq!(offset_of!(SurfaceTables, lives_by_level), 0x4420);
        assert_eq!(offset_of!(SurfaceTables, wheel_flags2), 0x4444);
        assert_eq!(offset_of!(SurfaceTables, wheel_flags), 0x4448);
        assert_eq!(offset_of!(SurfaceTables, colours), 0x444c);
    }
}
