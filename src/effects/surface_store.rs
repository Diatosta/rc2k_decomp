//! The surface's texture places and the emitters' lives and colours.

/// The surface's texture places and the emitters' lives and colours.
pub struct SurfaceTables {
    /// The cell places (0x40 cells of four floats, 16 bytes each), then the sub-cell places (0x40 x
    /// 0x100 bytes, 4 x 4 sub-cells of 16 bytes per cell). The cell draw reads cell 0x40, the first
    /// sub-cell's place.
    pub places: Places,
    /// The life by a wheel's flags' bits 6, 7 (`effects::emit_wheel_dust_cloud`).
    pub lives_by_flags: [u32; 4],
    /// The life by the level (`effects::emit_twin`; 9 entries are allowed, the last is the tint).
    pub lives_by_level: [u32; 9],
    /// The 128 colours a streak takes by its age (the other table has 257).
    pub colours: [u32; 128],
}

impl SurfaceTables {
    pub const fn new() -> Self {
        SurfaceTables {
            places: Places([0; 4352]),
            lives_by_flags: [0x0, 0xFA, 0x1F4, 0x3E8],
            lives_by_level: [0x7D, 0xFA, 0x177, 0x1F4, 0x28F, 0x2EE, 0x36B, 0x3E8, 0x0],
            colours: [
                0xFF0000, 0xFF0200, 0xFF0400, 0xFF0600, 0xFF0800, 0xFF0A00, 0xFF0C00, 0xFF0E00,
                0xFF1000, 0xFF1200, 0xFF1400, 0xFF1600, 0xFF1800, 0xFF1A00, 0xFF1C00, 0xFF1E00,
                0xFF2000, 0xFF2200, 0xFF2400, 0xFF2600, 0xFF2800, 0xFF2A00, 0xFF2C00, 0xFF2E00,
                0xFF3000, 0xFF3200, 0xFF3400, 0xFF3600, 0xFF3800, 0xFF3A00, 0xFF3C00, 0xFF3E00,
                0xFF4000, 0xFF4200, 0xFF4400, 0xFF4600, 0xFF4800, 0xFF4A00, 0xFF4C00, 0xFF4E00,
                0xFF5000, 0xFF5200, 0xFF5400, 0xFF5600, 0xFF5800, 0xFF5A00, 0xFF5C00, 0xFF5E00,
                0xFF6000, 0xFF6200, 0xFF6400, 0xFF6600, 0xFF6800, 0xFF6A00, 0xFF6C00, 0xFF6E00,
                0xFF7000, 0xFF7200, 0xFF7400, 0xFF7600, 0xFF7800, 0xFF7A00, 0xFF7C00, 0xFF7E00,
                0xFF8000, 0xFF8200, 0xFF8400, 0xFF8600, 0xFF8800, 0xFF8A00, 0xFF8C00, 0xFF8E00,
                0xFF9000, 0xFF9200, 0xFF9400, 0xFF9600, 0xFF9800, 0xFF9A00, 0xFF9C00, 0xFF9E00,
                0xFFA000, 0xFFA200, 0xFFA400, 0xFFA600, 0xFFA800, 0xFFAA00, 0xFFAC00, 0xFFAE00,
                0xFFB000, 0xFFB200, 0xFFB400, 0xFFB600, 0xFFB800, 0xFFBA00, 0xFFBC00, 0xFFBE00,
                0xFFC000, 0xFFC200, 0xFFC400, 0xFFC600, 0xFFC800, 0xFFCA00, 0xFFCC00, 0xFFCE00,
                0xFFD000, 0xFFD200, 0xFFD400, 0xFFD600, 0xFFD800, 0xFFDA00, 0xFFDC00, 0xFFDE00,
                0xFFE000, 0xFFE200, 0xFFE400, 0xFFE600, 0xFFE800, 0xFFEA00, 0xFFEC00, 0xFFEE00,
                0xFFF000, 0xFFF200, 0xFFF400, 0xFFF600, 0xFFF800, 0xFFFA00, 0xFFFC00, 0xFFFE00,
            ],
        }
    }
}

/// The places' bytes, kept as little-endian words (every access is a whole aligned word).
pub struct Places([u32; 4352]);

impl Places {
    /// The size in bytes.
    pub const fn len(&self) -> usize {
        4 * self.0.len()
    }

    /// The word at a byte offset (a multiple of 4, inside the table).
    pub fn get_u32(&self, offset: usize) -> u32 {
        self.0[offset / 4]
    }

    /// Sets the word at a byte offset (a multiple of 4, inside the table).
    pub fn set_u32(&mut self, offset: usize, value: u32) {
        self.0[offset / 4] = value;
    }
}
