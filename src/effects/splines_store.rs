//! The splines' tables and variables.

use crate::memory::session::{PlainBytes, PlainDword, PlainFlag, PlainFloat};

/// The car models' tables and the splines' head: the points and looks by car model
/// (`effects::emit_exhaust_smoke`, `effects::draw_mark_slots`, `effects::cars_particles`), then the
/// spline variables the looks table runs into (two dwords past it are read) and the spline file.
#[derive(rc2k_derive::Cells)]
pub struct SplineHead {
    /// Three floats by car model (34 models), a mark slot's point is an index into it (12 bytes
    /// each).
    pub points: [PlainFloat; 102],
    /// The looks by the car's model (two dwords 8 bytes apart, 33 pairs; models 32 and 33 read past
    /// the end: `found`, `wave`).
    pub looks: [PlainDword; 66],
    /// How many faces of the kind `gather_spline_points` found.
    pub found: PlainDword,
    /// The wave's angle (`effects::run_splines`).
    pub angle: PlainFloat,
    /// The wave (`effects::run_splines`).
    pub wave: PlainFloat,
    /// Added to the third table's highest point for its top fields.
    pub shift: [PlainDword; 3],
    /// 0 or 1, which of the two graded records is being placed.
    pub switch: PlainDword,
    /// The splines' fade-in counter, 0 .. 0xFF.
    pub made: PlainDword,
    /// `files\m\spline.dat` (0xC00 bytes at most): 12 bytes an entry, the blend weights.
    pub file: PlainBytes<3072>,
}

impl SplineHead {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineHead {
            points: [
                PlainFloat::new(2070.1),
                PlainFloat::new(265.6),
                PlainFloat::new(-572.5),
                PlainFloat::new(2055.2),
                PlainFloat::new(271.4),
                PlainFloat::new(-453.4),
                PlainFloat::new(2167.5),
                PlainFloat::new(229.9),
                PlainFloat::new(-463.5),
                PlainFloat::new(2232.2),
                PlainFloat::new(265.9),
                PlainFloat::new(472.5),
                PlainFloat::new(2204.0),
                PlainFloat::new(275.5),
                PlainFloat::new(427.5),
                PlainFloat::new(2012.3),
                PlainFloat::new(182.2),
                PlainFloat::new(-486.1),
                PlainFloat::new(2132.9),
                PlainFloat::new(296.0),
                PlainFloat::new(476.5),
                PlainFloat::new(2384.4),
                PlainFloat::new(236.3),
                PlainFloat::new(-547.2),
                PlainFloat::new(2184.4),
                PlainFloat::new(173.5),
                PlainFloat::new(-411.8),
                PlainFloat::new(2200.1),
                PlainFloat::new(252.0),
                PlainFloat::new(396.1),
                PlainFloat::new(2444.9),
                PlainFloat::new(259.8),
                PlainFloat::new(-458.9),
                PlainFloat::new(2019.6),
                PlainFloat::new(240.2),
                PlainFloat::new(-513.8),
                PlainFloat::new(2128.0),
                PlainFloat::new(232.8),
                PlainFloat::new(476.5),
                PlainFloat::new(1874.5),
                PlainFloat::new(244.1),
                PlainFloat::new(392.2),
                PlainFloat::new(2037.8),
                PlainFloat::new(203.6),
                PlainFloat::new(-538.7),
                PlainFloat::new(2243.2),
                PlainFloat::new(263.7),
                PlainFloat::new(-538.6),
                PlainFloat::new(2343.1),
                PlainFloat::new(319.1),
                PlainFloat::new(-501.5),
                PlainFloat::new(2135.3),
                PlainFloat::new(220.0),
                PlainFloat::new(-414.0),
                PlainFloat::new(1947.3),
                PlainFloat::new(340.3),
                PlainFloat::new(-548.3),
                PlainFloat::new(2029.5),
                PlainFloat::new(220.0),
                PlainFloat::new(-454.1),
                PlainFloat::new(2238.6),
                PlainFloat::new(299.3),
                PlainFloat::new(-264.7),
                PlainFloat::new(2024.5),
                PlainFloat::new(224.5),
                PlainFloat::new(405.7),
                PlainFloat::new(2365.3),
                PlainFloat::new(236.1),
                PlainFloat::new(-542.2),
                PlainFloat::new(1850.0),
                PlainFloat::new(350.0),
                PlainFloat::new(-400.0),
                PlainFloat::new(2060.0),
                PlainFloat::new(280.0),
                PlainFloat::new(480.0),
                PlainFloat::new(2120.0),
                PlainFloat::new(260.0),
                PlainFloat::new(-520.0),
                PlainFloat::new(1120.0),
                PlainFloat::new(180.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1120.0),
                PlainFloat::new(1120.0),
                PlainFloat::new(0.0),
                PlainFloat::new(950.0),
                PlainFloat::new(1600.0),
                PlainFloat::new(-300.0),
                PlainFloat::new(800.0),
                PlainFloat::new(1000.0),
                PlainFloat::new(-300.0),
                PlainFloat::new(2500.0),
                PlainFloat::new(250.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1450.0),
                PlainFloat::new(2800.0),
                PlainFloat::new(-0.0),
                PlainFloat::new(1450.0),
                PlainFloat::new(2800.0),
                PlainFloat::new(-0.0),
                PlainFloat::new(1929.0),
                PlainFloat::new(307.0),
                PlainFloat::new(-174.0),
            ],
            looks: [
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
            ],
            found: PlainDword::new(0x0),
            angle: PlainFloat::new(0.0),
            wave: PlainFloat::new(0.0),
            shift: [
                PlainDword::new(0x4000),
                PlainDword::new(0xFFFF8000),
                PlainDword::new(0x0),
            ],
            switch: PlainDword::new(0x0),
            made: PlainDword::new(0x0),
            file: PlainBytes::new([0; 3072]),
        }
    }
}

/// The splines' work places (`effects::bend_points`, `effects::splines`, `effects::place_spline`).
#[derive(rc2k_derive::Cells)]
pub struct SplineWork {
    /// Four points of three whole numbers: A, B, C and D (`effects::bend_points`).
    pub bend: [PlainDword; 12],
    /// The object being made.
    pub object: PlainDword,
    /// The next of 8 places.
    pub place: PlainDword,
    /// The place times 0xE90.
    pub offset: PlainDword,
    /// The vector the angle is measured from.
    pub up: [PlainFloat; 3],
    /// The points' size, as floats.
    pub size: [PlainFloat; 3],
    /// The X turn times the Y turn (a matrix of 12 floats).
    pub turn_xy: [PlainFloat; 12],
    /// All three turns.
    pub turn: [PlainFloat; 12],
    /// The turn about X.
    pub turn_x: [PlainFloat; 12],
    /// The turn about Y.
    pub turn_y: [PlainFloat; 12],
    /// The turn about Z.
    pub turn_z: [PlainFloat; 12],
    /// `order_spline_points`' work, one id a point (at most 0x80 points).
    pub ordered_ids: [PlainDword; 128],
    /// The ordered points, three whole numbers each.
    pub ordered_coords: [PlainDword; 384],
}

impl SplineWork {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineWork {
            bend: [const { PlainDword::new(0x0) }; 12],
            object: PlainDword::new(0x0),
            place: PlainDword::new(0x0),
            offset: PlainDword::new(0x0),
            up: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
            ],
            size: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            turn_xy: [const { PlainFloat::new(0.0) }; 12],
            turn: [const { PlainFloat::new(0.0) }; 12],
            turn_x: [const { PlainFloat::new(0.0) }; 12],
            turn_y: [const { PlainFloat::new(0.0) }; 12],
            turn_z: [const { PlainFloat::new(0.0) }; 12],
            ordered_ids: [const { PlainDword::new(0x0) }; 128],
            ordered_coords: [const { PlainDword::new(0x0) }; 384],
        }
    }
}

/// The four tables of eight spline records of 0xE90 bytes (`effects::splines`).
#[derive(rc2k_derive::Cells)]
pub struct SplineTables {
    /// Eight spline records.
    pub a: PlainBytes<29824>,
    /// Eight spline records, table B.
    pub b: PlainBytes<29824>,
    /// Eight spline records, table C.
    pub c: PlainBytes<29824>,
    /// Eight spline records, table D.
    pub d: PlainBytes<29824>,
}

impl SplineTables {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineTables {
            a: PlainBytes::new(crate::memory::session::with_runs::<29824>(&[
                (
                    0x38,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0xEC8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x1D58,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x2BE8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x3A78,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x4908,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x5798,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x6628,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
            ])),
            b: PlainBytes::new(crate::memory::session::with_runs::<29824>(&[
                (
                    0x38,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0xEC8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x1D58,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x2BE8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x3A78,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x4908,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x5798,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x6628,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
            ])),
            c: PlainBytes::new(crate::memory::session::with_runs::<29824>(&[
                (
                    0x38,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0xEC8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x1D58,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x2BE8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x3A78,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x4908,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x5798,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x6628,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
            ])),
            d: PlainBytes::new(crate::memory::session::with_runs::<29824>(&[
                (
                    0x38,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0xEC8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x1D58,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x2BE8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x3A78,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x4908,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x5798,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x6628,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
            ])),
        }
    }
}

/// The splines' values by kind (`effects::splines`).
#[derive(rc2k_derive::Cells)]
pub struct SplineKinds {
    /// The chosen kind, then the four kinds: the five values of the marks' kind.
    pub chosen: [PlainDword; 5],
    /// Five tables of 0x22 values by kind.
    pub by_kind: [PlainDword; 170],
    /// Set once the spline file is read.
    pub loaded: PlainFlag,
}

impl SplineKinds {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineKinds {
            chosen: [
                PlainDword::new(0x0),
                PlainDword::new(0x13),
                PlainDword::new(0x19),
                PlainDword::new(0x18),
                PlainDword::new(0x1C),
            ],
            by_kind: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x1),
                PlainDword::new(0x0),
                PlainDword::new(0x1),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x1),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x13),
                PlainDword::new(0x13),
                PlainDword::new(0xC),
                PlainDword::new(0x12),
                PlainDword::new(0x13),
                PlainDword::new(0x13),
                PlainDword::new(0x15),
                PlainDword::new(0x10),
                PlainDword::new(0x4),
                PlainDword::new(0x15),
                PlainDword::new(0x10),
                PlainDword::new(0xE),
                PlainDword::new(0x13),
                PlainDword::new(0x1C),
                PlainDword::new(0x13),
                PlainDword::new(0x15),
                PlainDword::new(0x11),
                PlainDword::new(0xE),
                PlainDword::new(0xA),
                PlainDword::new(0xB),
                PlainDword::new(0x11),
                PlainDword::new(0x16),
                PlainDword::new(0x10),
                PlainDword::new(0xE),
                PlainDword::new(0x15),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x13),
                PlainDword::new(0x18),
                PlainDword::new(0x18),
                PlainDword::new(0x1A),
                PlainDword::new(0x19),
                PlainDword::new(0x1C),
                PlainDword::new(0x18),
                PlainDword::new(0x1B),
                PlainDword::new(0x1C),
                PlainDword::new(0x18),
                PlainDword::new(0x1A),
                PlainDword::new(0x19),
                PlainDword::new(0x19),
                PlainDword::new(0x1A),
                PlainDword::new(0x12),
                PlainDword::new(0x18),
                PlainDword::new(0x18),
                PlainDword::new(0x1A),
                PlainDword::new(0x12),
                PlainDword::new(0xD),
                PlainDword::new(0xE),
                PlainDword::new(0x14),
                PlainDword::new(0x19),
                PlainDword::new(0x1C),
                PlainDword::new(0x11),
                PlainDword::new(0x1B),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x18),
                PlainDword::new(0x19),
                PlainDword::new(0x19),
                PlainDword::new(0x1B),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0x19),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0x19),
                PlainDword::new(0x19),
                PlainDword::new(0x18),
                PlainDword::new(0x1A),
                PlainDword::new(0x19),
                PlainDword::new(0x11),
                PlainDword::new(0x19),
                PlainDword::new(0x19),
                PlainDword::new(0x19),
                PlainDword::new(0x11),
                PlainDword::new(0xE),
                PlainDword::new(0xF),
                PlainDword::new(0x15),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0x12),
                PlainDword::new(0x1A),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x19),
                PlainDword::new(0x1C),
                PlainDword::new(0x1C),
                PlainDword::new(0x1C),
                PlainDword::new(0x1B),
                PlainDword::new(0x1F),
                PlainDword::new(0x1C),
                PlainDword::new(0x1C),
                PlainDword::new(0x1F),
                PlainDword::new(0x1B),
                PlainDword::new(0x1B),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0x1B),
                PlainDword::new(0x8),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0x1C),
                PlainDword::new(0x1C),
                PlainDword::new(0x1B),
                PlainDword::new(0x1A),
                PlainDword::new(0x17),
                PlainDword::new(0x1C),
                PlainDword::new(0x1F),
                PlainDword::new(0x14),
                PlainDword::new(0x1C),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x1C),
            ],
            loaded: PlainFlag::new(false),
        }
    }
}

/// The splines' file name (`effects::splines`).
#[derive(rc2k_derive::Cells)]
pub struct SplinePath {
    /// `files\m\spline.dat`, the file's name.
    pub path: PlainBytes<19>,
}

impl SplinePath {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplinePath {
            path: PlainBytes::new([
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x73, 0x70, 0x6C, 0x69, 0x6E, 0x65,
                0x2E, 0x64, 0x61, 0x74, 0x00,
            ]),
        }
    }
}

/// The splines' constants (`effects::run_splines`, `effects::place_spline`, `effects::splines`).
#[derive(rc2k_derive::Cells)]
pub struct SplineConstants {
    /// The wave's constants.
    pub wave_k: [PlainFloat; 3],
    /// `place_spline`'s factors and limits.
    pub place_k: [PlainFloat; 8],
    /// Each table's three values (four times three).
    pub angles: [PlainFloat; 12],
}

impl SplineConstants {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineConstants {
            wave_k: [
                PlainFloat::new(128.0),
                PlainFloat::new(0.2),
                PlainFloat::new(1.0),
            ],
            place_k: [
                PlainFloat::new(0.8),
                PlainFloat::new(0.4),
                PlainFloat::new(0.0),
                PlainFloat::new(0.6),
                PlainFloat::new(0.1),
                PlainFloat::new(0.0),
                PlainFloat::new(0.6),
                PlainFloat::new(0.1),
            ],
            angles: [const { PlainFloat::new(0.0) }; 12],
        }
    }
}

impl SplineHead {
    /// Dword `i` of the looks table: model 32 and 33 of the cars (34 models, pairs 8 bytes
    /// apart) run two and three dwords past the table's end into `found`, `angle` and `wave`
    /// (a slip kept as is).
    pub fn look(&self, i: usize) -> u32 {
        match i {
            0..66 => self.looks[i].get(),
            66 => self.found.get(),
            // Floats read as dwords, past the table.
            67 => self.angle.get().to_bits(),
            68 => self.wave.get().to_bits(),
            _ => panic!("the looks table is read at most up to `wave`"),
        }
    }
}
