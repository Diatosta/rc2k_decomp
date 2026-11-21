//! The cars' spec records: 34 records of 0xB0 bytes, one per car kind (`car_specs`, the struct
//! `CarSpec`: "00 renault maxi megane" .. "33 Renault Sport Clio"), read by
//! `physics::load_car_specs` (the work car's handling figures) and `race::race_shell` (the start
//! values). `light_records` follows.
//!

/// One car kind's figures.
#[cfg_attr(test, repr(C))]
pub struct CarSpec {
    /// The car's title, padded with spaces, e.g. "00 renault maxi megane".
    pub name: [u8; 24],
    /// The drive kind, checked 0 ..= 3 (`physics::load_car_specs`); also the start
    /// value (`race::race_shell`).
    pub drive_kind: u32,
    /// The gear count by its sign, held to 3 ..= 7 (`physics::load_car_specs`).
    pub gear_count: u32,
    /// A spec figure nothing reads; kept for the spec's byte layout.
    pub reserved_20: u32,
    /// Copied to the physics `grip_scale`, the factor `physics::drive_grip_forces`
    /// multiplies the driving grip force by.
    pub grip_scale: f32,
    /// Copied to the physics `side_grip_scale`, the factor `physics::drive_grip_forces`
    /// multiplies the sideways grip force by.
    pub side_grip_scale: f32,
    /// Copied to the physics `spec_surface` 0, 1, 2.
    pub surface: [f32; 3],
    /// Two figures times `PhysicsTables.gravity` into `PhysicsTables.spec_drive[0]`,
    /// `PhysicsTables.spec_drive[1]`.
    pub weights: [f32; 2],
    /// Three whole figures x 65536 / `Numbers.thousand` into `PhysicsTables.spec_ints[0]` ..
    /// (kinds from 0x17 on; the others take the game's constants).
    pub whole_figures: [u32; 3],
    /// Not read.
    pub reserved_4c: [u32; 4],
    /// Into `SceneDraw.scratch` (`scene::draw_store`'s `scratch`).
    pub scratch_value: u32,
    /// Into the physics `downforce_spec`.
    pub downforce: u32,
    /// Four whole figures as floats into `LightsAndGear.gear.figures[0]` .. (each at least
    /// 8.0).
    pub figures: [u32; 4],
    /// The engine's top, a whole number (8500 for the first); also the start
    /// value's limit (`CarSpecs.cars[0].engine_top`, `race::race_shell`).
    pub engine_top: u32,
    /// A whole figure divided by the top into `LightsAndGear.gear.drive_torque_figure`.
    pub over_top: u32,
    /// The torque curve, copied to the physics `spec_torque`.
    pub torque_curve: [u32; 9],
    /// Into the physics `ride_height_handling_scale` `physics::set_handling`
    /// multiplies the lerped ride-height handling value (`handling[4]`) by it.
    pub ride_height_handling_scale: u32,
    /// The shift points (`LightsAndGear.gear.shift_up_point` ..; the first x
    /// `PhysicsTables.spec_scale_a4`, the others + `Numbers.tenths[0]`).
    pub shift_points: [f32; 3],
}

#[cfg(test)]
const _: () = assert!(core::mem::size_of::<CarSpec>() == 0xB0);

/// The car kinds' records.
pub struct CarSpecs {
    pub cars: [CarSpec; 34],
}

/// The 34 records' starting values: the title, then the 38 dwords after it in the order of
/// `CarSpec`'s fields (floats by their bits).
#[rustfmt::skip]
const SPEC_ROWS: [(&[u8; 24], [u32; 38]); 34] = [
    (
        b"00 renault maxi megane  ",
        [
            0, 7, 0, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f75c28f, 0x3f800000, 0x3f800000,
            0x40100000, 0x794, 0x7da, 0x354, 0x3c0, 0x401f7cee, 0x3f028f5c, 0x40c00000, 0x3f000000,
            0x3dcccccd, 39, 109, 54, 146, 0x2134, 206, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"01 Vauxhall Astra       ",
        [
            0, 6, 0, 0x3f7851ec, 0x3f8ccccd, 0x3f800000, 0x3f70a3d7, 0x3f733333, 0x3f800000,
            0x40066666, 0x7a4, 0x7a4, 0x375, 0x3c0, 0x402872b0, 0x3f0ccccd, 0x40c00000, 0x3f000000,
            0x3dcccccd, 38, 107, 52, 143, 0x2198, 194, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"02 Seat Cordoba         ",
        [
            1, 6, 0, 0x3f666666, 0x3f866666, 0x3f800000, 0x3f7851ec, 0x3f7851ec, 0x3f800000,
            0x3ff9999a, 0x7c4, 0x7fb, 0x349, 0x4ce, 0x401c5a1d, 0x3f2147ae, 0x40c00000, 0x3f000000,
            0x3dcccccd, 47, 110, 58, 143, 0x17d4, 244, 0x3ecccccd, 0x3f000000, 0x3f19999a,
            0x3f4ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f666666, 0x3f19999a, 0x3f800000,
            0x3f5eb852, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"03 Hyundai Coupe        ",
        [
            0, 6, 0, 0x3f800000, 0x3f866666, 0x3f800000, 0x3f733333, 0x3f6b851f, 0x3f800000,
            0x40100000, 0x853, 0x81c, 0x36a, 0x406, 0x401e6666, 0x3f0a3d71, 0x40c00000, 0x3f000000,
            0x3dcccccd, 38, 105, 50, 141, 0x203a, 187, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"04 Subaru Impreza       ",
        [
            1, 6, 0, 0x3f800000, 0x3f99999a, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x3feccccd, 0x83d, 0x83d, 0x35f, 0x4ce, 0x402147ae, 0x3f0a3d71, 0x40c00000, 0x3f000000,
            0x3dcccccd, 49, 113, 59, 147, 0x186a, 255, 0x3ecccccd, 0x3f000000, 0x3f19999a,
            0x3f4ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f666666, 0x3f19999a, 0x3f800000,
            0x3f5eb852, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"05 Peugeot 106          ",
        [
            0, 6, 0, 0x3f800000, 0x3f933333, 0x3f800000, 0x3f68f5c3, 0x3f7851ec, 0x3f800000,
            0x40100000, 0x736, 0x76d, 0x36a, 0x37a, 0x4018f5c3, 0x3f028f5c, 0x40c00000, 0x3f000000,
            0x3dcccccd, 37, 94, 47, 131, 0x20d0, 155, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"06 Citroen Maxo         ",
        [
            1, 6, 0, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x40100000, 0x74f, 0x759, 0x37f, 0x370, 0x401a3d71, 0x3f028f5c, 0x40c00000, 0x3f000000,
            0x3dcccccd, 50, 110, 60, 150, 0x1964, 0x113, 0x3ecccccd, 0x3f000000, 0x3f19999a,
            0x3f4ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f666666, 0x3f19999a, 0x3f800000,
            0x3f5eb852, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"07 Mitsubishi Lancer    ",
        [
            1, 6, 0, 0x3f6b851f, 0x3f800000, 0x3f800000, 0x3f7d70a4, 0x3f828f5c, 0x3f800000,
            0x3ff9999a, 0x81c, 0x8d6, 0x375, 0x4ce, 0x4020a3d7, 0x3f147ae1, 0x40c00000, 0x3f000000,
            0x3dcccccd, 50, 115, 60, 152, 0x1838, 0x10e, 0x3ecccccd, 0x3f000000, 0x3f19999a,
            0x3f4ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f666666, 0x3f19999a, 0x3f800000,
            0x3f5eb852, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"08 VW Golf              ",
        [
            0, 6, 0, 0x3f7851ec, 0x3f828f5c, 0x3f800000, 0x3f733333, 0x3f75c28f, 0x3f800000,
            0x400ccccd, 0x78e, 0x832, 0x375, 0x3d4, 0x40215810, 0x3f07ae14, 0x40c00000, 0x3f000000,
            0x3dcccccd, 37, 106, 50, 140, 0x2328, 184, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"09 Nissan Almera        ",
        [
            0, 6, 0, 0x3f6b851f, 0x3f800000, 0x3f800000, 0x3f68f5c3, 0x3f733333, 0x3f800000,
            0x40133333, 0x7da, 0x811, 0x380, 0x3c0, 0x40223d71, 0x3f0ccccd, 0x40c00000, 0x3f000000,
            0x3dcccccd, 38, 104, 49, 140, 0x2328, 184, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"10 Proton Wira          ",
        [
            1, 6, 0, 0x3f6b851f, 0x3f800000, 0x3f800000, 0x3f6b851f, 0x3f6b851f, 0x3f800000,
            0x3ff9999a, 0x848, 0x8cb, 0x38b, 0x4e7, 0x40200000, 0x3f1eb852, 0x40c00000, 0x3f000000,
            0x3dcccccd, 47, 109, 57, 140, 0x1770, 238, 0x3ecccccd, 0x3f000000, 0x3f19999a,
            0x3f4ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f666666, 0x3f19999a, 0x3f800000,
            0x3f5eb852, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"11 Citroen Saxo         ",
        [
            0, 6, 0, 0x3f800000, 0x3f8ccccd, 0x3f800000, 0x3f6b851f, 0x3f7d70a4, 0x3f800000,
            0x4009999a, 0x74f, 0x759, 0x37f, 0x370, 0x401a3d71, 0x3f028f5c, 0x40c00000, 0x3f000000,
            0x3dcccccd, 37, 99, 48, 134, 0x20d0, 164, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"12 Honda Civic          ",
        [
            0, 6, 0, 0x3f75c28f, 0x3f828f5c, 0x3f800000, 0x3f666666, 0x3f75c28f, 0x3f800000,
            0x40000000, 0x841, 0x7c7, 0x380, 0x3f2, 0x4030a3d7, 0x3f0ccccd, 0x40c00000, 0x3f000000,
            0x3dcccccd, 38, 100, 50, 135, 0x24b8, 168, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"13 Nissan Micra         ",
        [
            0, 6, 0, 0x3f7ae148, 0x3f866666, 0x3f800000, 0x3f63d70a, 0x3f7d70a4, 0x3f800000,
            0x40133333, 0x74e, 0x6c9, 0x306, 0x366, 0x401ccccd, 0x3f000000, 0x40c00000, 0x3f000000,
            0x3dcccccd, 35, 93, 45, 127, 0x238c, 148, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"14 Peugeot 206          ",
        [
            1, 6, 0, 0x3f800000, 0x3f8ccccd, 0x3f800000, 0x3f7ae148, 0x3f800000, 0x3f800000,
            0x3feccccd, 0x84c, 0x79b, 0x36a, 0x4ce, 0x401df3b6, 0x3f0ccccd, 0x40c00000, 0x3f000000,
            0x3dcccccd, 50, 114, 59, 150, 0x1806, 0x105, 0x3ecccccd, 0x3f000000, 0x3f19999a,
            0x3f4ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f666666, 0x3f19999a, 0x3f800000,
            0x3f5eb852, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"15 Ford Escort RS2000   ",
        [
            0, 6, 0, 0x3f7d70a4, 0x3f933333, 0x3f800000, 0x3f6b851f, 0x3f68f5c3, 0x3f800000,
            0x40066666, 0x7bc, 0x84c, 0x35f, 0x3c0, 0x4021eb85, 0x3f0f5c29, 0x40c00000, 0x3f000000,
            0x3dcccccd, 36, 102, 47, 136, 0x1fa4, 174, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"16 Skoda Octavia        ",
        [
            0, 6, 0, 0x3f666666, 0x3f828f5c, 0x3f800000, 0x3f6e147b, 0x3f666666, 0x3f800000,
            0x40133333, 0x848, 0x8f9, 0x367, 0x3d4, 0x4020c49c, 0x3f1eb852, 0x40c00000, 0x3f000000,
            0x3dcccccd, 38, 103, 48, 139, 0x206c, 177, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"17 Seat Ibiza           ",
        [
            0, 6, 0, 0x3f7ae148, 0x3f8ccccd, 0x3f800000, 0x3f70a3d7, 0x3f733333, 0x3f800000,
            0x40166666, 0x79a, 0x6f8, 0x35d, 0x3c5, 0x401c3958, 0x3f0a3d71, 0x40c00000, 0x3f000000,
            0x3dcccccd, 37, 104, 49, 139, 0x251c, 181, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"18 Skoda Felicia        ",
        [
            0, 6, 0, 0x3f7851ec, 0x3f800000, 0x3f800000, 0x3f6147ae, 0x3f75c28f, 0x3f800000,
            0x40100000, 0x7b9, 0x735, 0x349, 0x352, 0x401ccccd, 0x3f051eb8, 0x40c00000, 0x3f000000,
            0x3dcccccd, 35, 93, 45, 127, 0x1f40, 152, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"19 Proton Compact       ",
        [
            0, 6, 0, 0x3f733333, 0x3f866666, 0x3f800000, 0x3f666666, 0x3f733333, 0x3f800000,
            0x40100000, 0x800, 0x767, 0x35d, 0x375, 0x401c28f6, 0x3f0ccccd, 0x40c00000, 0x3f000000,
            0x3dcccccd, 36, 91, 44, 127, 0x2134, 146, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"20 Ford Maxi            ",
        [
            0, 6, 0, 0x3f7851ec, 0x3f828f5c, 0x3f800000, 0x3f6e147b, 0x3f70a3d7, 0x3f800000,
            0x4009999a, 0x7b4, 0x868, 0x392, 0x3c0, 0x4021eb85, 0x3f0ccccd, 0x40c00000, 0x3f000000,
            0x3dcccccd, 37, 103, 49, 138, 0x2198, 178, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"21 Ford Puma            ",
        [
            0, 6, 0, 0x3f7ae148, 0x3f8ccccd, 0x3f800000, 0x3f6b851f, 0x3f7851ec, 0x3f800000,
            0x4019999a, 0x7c0, 0x793, 0x370, 0x316, 0x401c28f6, 0x3f07ae14, 0x40c00000, 0x3f000000,
            0x3dcccccd, 40, 90, 50, 126, 0x2134, 150, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"22 Mitsubishi EVO 4     ",
        [
            1, 6, 0, 0x3f6b851f, 0x3f800000, 0x3f800000, 0x3f70a3d7, 0x3f733333, 0x3f800000,
            0x3ff33333, 0x82f, 0x8c0, 0x338, 0x4ce, 0x4020a3d7, 0x3f147ae1, 0x40c00000, 0x3f000000,
            0x3dcccccd, 48, 111, 58, 144, 0x1838, 249, 0x3ecccccd, 0x3f000000, 0x3f19999a,
            0x3f4ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f666666, 0x3f19999a, 0x3f800000,
            0x3f5eb852, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"23 MF Supercar          ",
        [
            1, 7, 0, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x400ccccd, 0x7f0, 0x81c, 0x375, 0x4e2, 0x401f0a3d, 0x3f0ccccd, 0x40c00000, 0x3f000000,
            0x3dcccccd, 55, 130, 65, 170, 0x1b58, 0x140, 0x3ecccccd, 0x3f000000, 0x3f19999a,
            0x3f4ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f666666, 0x3f19999a, 0x3f800000,
            0x3f5eb852, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"24 MF Boyota            ",
        [
            1, 6, 0, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x400ccccd, 0x7d0, 0x7d0, 0x3e8, 0x3e8, 0x4028f5c3, 0x3e800000, 0x40c00000, 0x3f000000,
            0x3dcccccd, 50, 125, 60, 160, 0x1a90, 0x12c, 0x3ecccccd, 0x3f000000, 0x3f19999a,
            0x3f4ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f666666, 0x3f19999a, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"25 Pug 309              ",
        [
            0, 5, 0, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x400ccccd, 0x7d0, 0x7d0, 0x3e8, 0x3e8, 0x4028f5c3, 0x3e800000, 0x40c00000, 0x3f000000,
            0x3dcccccd, 35, 90, 45, 125, 0x1d4c, 150, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"26 Skip                 ",
        [
            0, 5, 0, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x400ccccd, 0x7d0, 0x708, 0x3e8, 0x3e8, 0x4028f5c3, 0x3e800000, 0x40c00000, 0x3f000000,
            0x3dcccccd, 30, 70, 40, 90, 0x1964, 80, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8, 0x3f23d70a,
            0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f000000, 0x3f733333,
            0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"27 Potato               ",
        [
            1, 6, 0, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x400ccccd, 0x3e8, 0x3e8, 0x3e8, 0x3e8, 0x4028f5c3, 0x3e800000, 0x40c00000, 0x3f000000,
            0x3dcccccd, 40, 110, 48, 140, 0x157c, 240, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f99999a,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"28 Cow                  ",
        [
            0, 6, 0, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x400ccccd, 0x3e8, 0x546, 0x28a, 0x3e8, 0x4028f5c3, 0x3e800000, 0x40c00000, 0x3f000000,
            0x3dcccccd, 35, 90, 45, 120, 0x1f40, 140, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0, 0x3f733333,
            0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"29 Sheep                ",
        [
            0, 5, 0, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x400ccccd, 0x320, 0x320, 0x2bc, 0x3e8, 0x4028f5c3, 0x3e800000, 0x40c00000, 0x3f000000,
            0x3dcccccd, 35, 85, 40, 110, 0x1d4c, 125, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"30 RC Ibiza             ",
        [
            0, 5, 0, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x400ccccd, 0xa8c, 0xa8c, 0x3e8, 0x3e8, 0x4028f5c3, 0x3e800000, 0x40c00000, 0x3f000000,
            0x3dcccccd, 40, 100, 45, 130, 0x2134, 160, 0x3ecccccd, 0x3f000000, 0x3f19999a,
            0x3f4ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f666666, 0x3f19999a, 0x3f000000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"31 Volvo Estate         ",
        [
            0, 5, 0, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x400ccccd, 0xa8c, 0xa8c, 0x3e8, 0x3e8, 0x4028f5c3, 0x3e800000, 0x40c00000, 0x3f000000,
            0x3dcccccd, 40, 100, 45, 130, 0x2134, 160, 0x3ecccccd, 0x3f000000, 0x3f19999a,
            0x3f4ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f666666, 0x3f19999a, 0x3f000000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"32 Local Shop           ",
        [
            0, 5, 0, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x400ccccd, 0x640, 0x708, 0x384, 0x3e8, 0x4028f5c3, 0x3e800000, 0x40c00000, 0x3f000000,
            0x3dcccccd, 30, 75, 40, 100, 0x1964, 120, 0x3ecccccd, 0x3ee147ae, 0x3f051eb8,
            0x3f23d70a, 0x3f4ccccd, 0x3f6b851f, 0x3f7ae148, 0x3f800000, 0x3f7ae148, 0x3fc00000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
    (
        b"33 Renault Sport Clio   ",
        [
            1, 6, 0, 0x3f800000, 0x3f8ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
            0x400ccccd, 0x76c, 0x76c, 0x36b, 0x44c, 0x401c28f6, 0x3f000000, 0x40c00000, 0x3f000000,
            0x3dcccccd, 50, 125, 60, 160, 0x2134, 0x12c, 0x3ecccccd, 0x3f000000, 0x3f19999a,
            0x3f4ccccd, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f666666, 0x3f19999a, 0x3f800000,
            0x3f733333, 0x3f19999a, 0x3f333333,
        ],
    ),
];

impl CarSpec {
    /// A record from its title and the 38 dwords after it (`SPEC_ROWS`, floats by their bits).
    const fn from_row(name: &[u8; 24], d: &[u32; 38]) -> Self {
        Self {
            name: *name,
            drive_kind: d[0],
            gear_count: d[1],
            reserved_20: d[2],
            grip_scale: f32::from_bits(d[3]),
            side_grip_scale: f32::from_bits(d[4]),
            surface: [
                f32::from_bits(d[5]),
                f32::from_bits(d[6]),
                f32::from_bits(d[7]),
            ],
            weights: [f32::from_bits(d[8]), f32::from_bits(d[9])],
            whole_figures: [d[10], d[11], d[12]],
            reserved_4c: [d[13], d[14], d[15], d[16]],
            scratch_value: d[17],
            downforce: d[18],
            figures: [d[19], d[20], d[21], d[22]],
            engine_top: d[23],
            over_top: d[24],
            torque_curve: [
                d[25], d[26], d[27], d[28], d[29], d[30], d[31], d[32], d[33],
            ],
            ride_height_handling_scale: d[34],
            shift_points: [
                f32::from_bits(d[35]),
                f32::from_bits(d[36]),
                f32::from_bits(d[37]),
            ],
        }
    }
}

/// The piece, with the image's first values.
impl CarSpecs {
    pub const fn new() -> Self {
        let mut cars = [const { CarSpec::from_row(&[0; 24], &[0; 38]) }; 34];
        let mut n = 0;
        while n < 34 {
            let (name, dwords) = &SPEC_ROWS[n];
            cars[n] = CarSpec::from_row(name, dwords);
            n += 1;
        }
        Self { cars }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A record's bytes in the file's order.
    fn record_bytes(spec: &CarSpec) -> Vec<u8> {
        let mut bytes = spec.name.to_vec();
        let words = [spec.drive_kind, spec.gear_count, spec.reserved_20]
            .into_iter()
            .chain([spec.grip_scale, spec.side_grip_scale].map(f32::to_bits))
            .chain(spec.surface.map(f32::to_bits))
            .chain(spec.weights.map(f32::to_bits))
            .chain(spec.whole_figures)
            .chain(spec.reserved_4c)
            .chain([spec.scratch_value, spec.downforce])
            .chain(spec.figures)
            .chain([spec.engine_top, spec.over_top])
            .chain(spec.torque_curve)
            .chain([spec.ride_height_handling_scale])
            .chain(spec.shift_points.map(f32::to_bits));
        for word in words {
            bytes.extend(word.to_le_bytes());
        }
        bytes
    }

    /// The records' whole bytes, as a checksum: a wrong dword in a row shows here.
    #[test]
    fn the_car_specs_keep_the_bytes_of_the_image() {
        let bytes: Vec<u8> = CarSpecs::new().cars.iter().flat_map(record_bytes).collect();
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in &bytes {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
        assert_eq!(bytes.len(), 34 * 0xB0);
        assert_eq!(hash, 0xc73d_3061_7688_2fa8);
    }

    #[test]
    fn the_cars_start_as_the_games() {
        let physics = &crate::test_support::own::race().physics;
        let title = |k: usize| physics.car_specs().cars[k].name[..9].to_vec();
        assert_eq!(title(0), b"00 renaul");
        assert_eq!(title(33), b"33 Renaul");
        assert_eq!(physics.car_specs().cars[0].engine_top, 8500);
    }
}
