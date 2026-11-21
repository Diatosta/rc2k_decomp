//! The splines' tables and variables.

/// The car models' tables and the splines' head: the points and looks by car model
/// (`effects::emit_exhaust_smoke`, `effects::draw_mark_slots`, `effects::cars_particles`), then the
/// spline variables the looks table runs into (two dwords past it are read) and the spline file.
pub struct SplineHead {
    /// Three floats by car model (34 models), a mark slot's point is an index into it (12 bytes
    /// each).
    pub points: [f32; 102],
    /// The looks by the car's model (two dwords 8 bytes apart, 33 pairs; models 32 and 33 read past
    /// the end: `found`, `wave`).
    pub looks: [u32; 66],
    /// How many faces of the kind `gather_spline_points` found.
    pub found: u32,
    /// The wave's angle (`effects::run_splines`).
    pub angle: f32,
    /// The wave (`effects::run_splines`).
    pub wave: f32,
    /// Added to the third table's highest point for its top fields.
    pub shift: [u32; 3],
    /// 0 or 1, which of the two graded records is being placed.
    pub switch: u32,
    /// The splines' fade-in counter, 0 .. 0xFF.
    pub made: u32,
    /// `files\m\spline.dat` (0xC00 bytes at most): 12 bytes an entry, the blend weights.
    pub file: [u8; 3072],
}

impl SplineHead {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineHead {
            points: [
                2070.1, 265.6, -572.5, 2055.2, 271.4, -453.4, 2167.5, 229.9, -463.5, 2232.2, 265.9,
                472.5, 2204.0, 275.5, 427.5, 2012.3, 182.2, -486.1, 2132.9, 296.0, 476.5, 2384.4,
                236.3, -547.2, 2184.4, 173.5, -411.8, 2200.1, 252.0, 396.1, 2444.9, 259.8, -458.9,
                2019.6, 240.2, -513.8, 2128.0, 232.8, 476.5, 1874.5, 244.1, 392.2, 2037.8, 203.6,
                -538.7, 2243.2, 263.7, -538.6, 2343.1, 319.1, -501.5, 2135.3, 220.0, -414.0,
                1947.3, 340.3, -548.3, 2029.5, 220.0, -454.1, 2238.6, 299.3, -264.7, 2024.5, 224.5,
                405.7, 2365.3, 236.1, -542.2, 1850.0, 350.0, -400.0, 2060.0, 280.0, 480.0, 2120.0,
                260.0, -520.0, 1120.0, 180.0, 0.0, 1120.0, 1120.0, 0.0, 950.0, 1600.0, -300.0,
                800.0, 1000.0, -300.0, 2500.0, 250.0, 0.0, 1450.0, 2800.0, -0.0, 1450.0, 2800.0,
                -0.0, 1929.0, 307.0, -174.0,
            ],
            looks: [
                0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800,
                0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000,
                0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800,
                0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000,
                0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800,
                0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000,
                0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800, 0x14000, 0xD800,
                0x14000, 0xD800, 0x14000,
            ],
            found: 0x0,
            angle: 0.0,
            wave: 0.0,
            shift: [0x4000, 0xFFFF8000, 0x0],
            switch: 0x0,
            made: 0x0,
            file: [0; 3072],
        }
    }
}

/// The splines' work places (`effects::bend_points`, `effects::splines`, `effects::place_spline`).
pub struct SplineWork {
    /// The next of 8 places.
    pub place: u32,
    /// The vector the angle is measured from.
    pub up: [f32; 3],
    /// The points' size, as floats.
    pub size: [f32; 3],
    /// The turns about X, Y and Z multiplied, as the last record's points were turned.
    pub turn: [f32; 9],
    /// `order_spline_points`' work, one id a point (at most 0x80 points).
    pub ordered_ids: [u32; 128],
    /// The ordered points, three whole numbers each.
    pub ordered_coords: [u32; 384],
}

impl SplineWork {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineWork {
            place: 0x0,
            up: [0.0, 0.0, 1.0],
            size: [0.0, 0.0, 0.0],
            turn: [0.0; 9],
            ordered_ids: [const { 0x0 }; 128],
            ordered_coords: [const { 0x0 }; 384],
        }
    }
}

/// The four tables of eight spline records of 0xE90 bytes (`effects::splines`), end to end.
pub struct SplineTables {
    /// Tables A .. D, eight spline records each.
    pub tables: [[u8; 29824]; 4],
}

impl SplineTables {
    /// The values the game starts with: each record's graded heights and top fields.
    pub const fn new() -> Self {
        const HEADS: [u8; 20] = [
            0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
            0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
        ];
        let table = crate::memory::session::with_runs::<29824>(&[
            (0x38, &HEADS),
            (0xEC8, &HEADS),
            (0x1D58, &HEADS),
            (0x2BE8, &HEADS),
            (0x3A78, &HEADS),
            (0x4908, &HEADS),
            (0x5798, &HEADS),
            (0x6628, &HEADS),
        ]);
        SplineTables { tables: [table; 4] }
    }
}

/// The splines' values by kind (`effects::splines`).
pub struct SplineKinds {
    /// The chosen kind, then the four kinds: the five values of the marks' kind.
    pub chosen: [u32; 5],
    /// Five tables of 0x22 values by kind.
    pub by_kind: [u32; 170],
    /// Set once the spline file is read.
    pub loaded: bool,
}

impl SplineKinds {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineKinds {
            chosen: [0x0, 0x13, 0x19, 0x18, 0x1C],
            by_kind: [
                0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0, 0x0,
                0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                0x0, 0x0, 0x13, 0x13, 0xC, 0x12, 0x13, 0x13, 0x15, 0x10, 0x4, 0x15, 0x10, 0xE,
                0x13, 0x1C, 0x13, 0x15, 0x11, 0xE, 0xA, 0xB, 0x11, 0x16, 0x10, 0xE, 0x15,
                0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
                0xFFFFFFFF, 0x13, 0x18, 0x18, 0x1A, 0x19, 0x1C, 0x18, 0x1B, 0x1C, 0x18, 0x1A, 0x19,
                0x19, 0x1A, 0x12, 0x18, 0x18, 0x1A, 0x12, 0xD, 0xE, 0x14, 0x19, 0x1C, 0x11, 0x1B,
                0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
                0xFFFFFFFF, 0x18, 0x19, 0x19, 0x1B, 0x1A, 0x1B, 0x19, 0x1A, 0x1B, 0x19, 0x19, 0x18,
                0x1A, 0x19, 0x11, 0x19, 0x19, 0x19, 0x11, 0xE, 0xF, 0x15, 0x1A, 0x1B, 0x12, 0x1A,
                0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
                0xFFFFFFFF, 0x19, 0x1C, 0x1C, 0x1C, 0x1B, 0x1F, 0x1C, 0x1C, 0x1F, 0x1B, 0x1B, 0x1A,
                0x1B, 0x1B, 0x8, 0x1A, 0x1B, 0x1C, 0x1C, 0x1B, 0x1A, 0x17, 0x1C, 0x1F, 0x14, 0x1C,
                0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
                0xFFFFFFFF, 0x1C,
            ],
            loaded: false,
        }
    }
}

/// The splines' file name (`effects::splines`).
pub struct SplinePath {
    /// `files\m\spline.dat`, the file's name.
    pub path: [u8; 19],
}

impl SplinePath {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplinePath {
            path: [
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x73, 0x70, 0x6C, 0x69, 0x6E, 0x65,
                0x2E, 0x64, 0x61, 0x74, 0x00,
            ],
        }
    }
}

/// The splines' constants (`effects::run_splines`, `effects::place_spline`, `effects::splines`).
pub struct SplineConstants {
    /// The wave's constants.
    pub wave_k: [f32; 3],
    /// `place_spline`'s factors and limits.
    pub place_k: [f32; 8],
    /// Each table's three values (four times three).
    pub angles: [f32; 12],
}

impl SplineConstants {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineConstants {
            wave_k: [128.0, 0.2, 1.0],
            place_k: [0.8, 0.4, 0.0, 0.6, 0.1, 0.0, 0.6, 0.1],
            angles: [const { 0.0 }; 12],
        }
    }
}

impl SplineHead {
    /// Dword `i` of the looks table: model 32 and 33 of the cars (34 models, pairs 8 bytes
    /// apart) run two and three dwords past the table's end into `found`, `angle` and `wave`
    /// (a slip kept as is).
    pub fn look(&self, i: usize) -> u32 {
        match i {
            0..66 => self.looks[i],
            66 => self.found,
            // Floats read as dwords, past the table.
            67 => self.angle.to_bits(),
            68 => self.wave.to_bits(),
            _ => panic!("the looks table is read at most up to `wave`"),
        }
    }
}
