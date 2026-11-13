//! The cars' spec records, 0x5C790C .. 0x5C906C: 34 records of 0xB0 bytes, one per car kind
//! (`g_carSpecs`, IDA struct `CarSpec`: "00 renault maxi megane" .. "33 Renault Sport Clio"), read by
//! `physics::load_car_specs` (the work car's handling figures) and `race::race_shell` (the
//! start values). `g_lightRecords` follows.
//! Names are readings unless IDA's.

use crate::cells::ZeroCells;

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{PlainBytes, PlainDword, PlainFloat};

/// One car kind's figures (IDA struct `CarSpec`).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct CarSpec {
    /// +0x0: the car's title, padded with spaces, e.g. "00 renault maxi megane"
    /// (drawn by 0x4426D4).
    pub name: PlainBytes<24>,
    /// +0x18: the drive kind, checked 0 ..= 3 (`physics::load_car_specs`); also the start
    /// value (`race::race_shell`).
    pub drive_kind: PlainDword,
    /// +0x1C: the gear count by its sign, held to 3 ..= 7 (`physics::load_car_specs`).
    pub gear_count: PlainDword,
    /// +0x20: copied to `PhysicsTables.unused_spec_20` (`unused_spec_20`) by `physics::load_car_specs`; nothing reads that copy (IDA: one write at 0x421692, no reader), so the figure is unused.
    pub unused_20: PlainDword,
    /// +0x24: copied to the physics `grip_scale`, the factor `physics::drive_grip_forces` multiplies the driving grip force by.
    pub grip_scale: PlainFloat,
    /// +0x28: copied to the physics `side_grip_scale`, the factor `physics::drive_grip_forces` multiplies the sideways grip force by.
    pub side_grip_scale: PlainFloat,
    /// +0x2C .. +0x34: copied to the physics `spec_surface` 0, 1, 2 (+0x2C, +0x30, +0x34).
    pub surface: [PlainFloat; 3],
    /// +0x38, +0x3C: two figures times `PhysicsTables.gravity` into `PhysicsTables.spec_drive[0]`, `PhysicsTables.spec_drive[1]`.
    pub weights: [PlainFloat; 2],
    /// +0x40: three whole figures x 65536 / `Numbers.thousand` into `PhysicsTables.spec_ints[0]` .. (kinds
    /// from 0x17 on; the others take the game's constants).
    pub whole_figures: [PlainDword; 3],
    /// +0x4C: not read by the game's code.
    pub unused_4c: [PlainDword; 4],
    /// +0x5C: into `SceneDraw.scratch` (`scene::draw_store`'s `scratch`).
    pub scratch_value: PlainDword,
    /// +0x60: into the physics `downforce_spec`.
    pub downforce: PlainDword,
    /// +0x64: four whole figures as floats into `LightsAndGear.gear.figures[0]` .. (each at least 8.0).
    pub figures: [PlainDword; 4],
    /// +0x74: the engine's top, a whole number (8500 for the first); also the start
    /// value's limit (`CarSpecs.cars[0].engine_top`, `race::race_shell`).
    pub engine_top: PlainDword,
    /// +0x78: a whole figure divided by the top into `LightsAndGear.gear.drive_torque_figure`.
    pub over_top: PlainDword,
    /// +0x7C: the torque curve, copied to the physics `spec_torque`.
    pub torque_curve: [PlainDword; 9],
    /// +0xA0: into the physics `ride_height_handling_scale` (`g_specA0`, 0x614C18): `physics::set_handling` multiplies the lerped ride-height handling value (`handling[4]`) by it.
    pub ride_height_handling_scale: PlainDword,
    /// +0xA4: the shift points (`LightsAndGear.gear.shift_up_point` ..; the first x `PhysicsTables.spec_scale_a4`, the others +
    /// `Numbers.tenths[0]`).
    pub shift_points: [PlainFloat; 3],
}

const _: () = assert!(core::mem::size_of::<CarSpec>() == 0xB0);

/// The car kinds' records.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct CarSpecs {
    pub cars: [CarSpec; 34],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl CarSpecs {
    pub const fn new() -> Self {
        Self {
            cars: {
                let mut a: [CarSpec; 34] = ZeroCells::ZERO;
                a[0] = CarSpec {
                    name: PlainBytes::new(*b"00 renault maxi megane  "),
                    gear_count: PlainDword::new(7),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.96),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.25)],
                    whole_figures: [
                        PlainDword::new(0x794),
                        PlainDword::new(0x7DA),
                        PlainDword::new(0x354),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3C0),
                        PlainDword::new(0x401F7CEE),
                        PlainDword::new(0x3F028F5C),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x27),
                        PlainDword::new(0x6D),
                        PlainDword::new(0x36),
                        PlainDword::new(0x92),
                    ],
                    engine_top: PlainDword::new(0x2134),
                    over_top: PlainDword::new(0xCE),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[1] = CarSpec {
                    name: PlainBytes::new(*b"01 Vauxhall Astra       "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.97),
                    side_grip_scale: PlainFloat::new(1.1),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.94),
                        PlainFloat::new(0.95),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.1)],
                    whole_figures: [
                        PlainDword::new(0x7A4),
                        PlainDword::new(0x7A4),
                        PlainDword::new(0x375),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3C0),
                        PlainDword::new(0x402872B0),
                        PlainDword::new(0x3F0CCCCD),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x26),
                        PlainDword::new(0x6B),
                        PlainDword::new(0x34),
                        PlainDword::new(0x8F),
                    ],
                    engine_top: PlainDword::new(0x2198),
                    over_top: PlainDword::new(0xC2),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[2] = CarSpec {
                    name: PlainBytes::new(*b"02 Seat Cordoba         "),
                    drive_kind: PlainDword::new(1),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.9),
                    side_grip_scale: PlainFloat::new(1.05),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.97),
                        PlainFloat::new(0.97),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(1.95)],
                    whole_figures: [
                        PlainDword::new(0x7C4),
                        PlainDword::new(0x7FB),
                        PlainDword::new(0x349),
                    ],
                    unused_4c: [
                        PlainDword::new(0x4CE),
                        PlainDword::new(0x401C5A1D),
                        PlainDword::new(0x3F2147AE),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x2F),
                        PlainDword::new(0x6E),
                        PlainDword::new(0x3A),
                        PlainDword::new(0x8F),
                    ],
                    engine_top: PlainDword::new(0x17D4),
                    over_top: PlainDword::new(0xF4),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x3F19999A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F666666),
                        PlainDword::new(0x3F19999A),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.87),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[3] = CarSpec {
                    name: PlainBytes::new(*b"03 Hyundai Coupe        "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.05),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.92),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.25)],
                    whole_figures: [
                        PlainDword::new(0x853),
                        PlainDword::new(0x81C),
                        PlainDword::new(0x36A),
                    ],
                    unused_4c: [
                        PlainDword::new(0x406),
                        PlainDword::new(0x401E6666),
                        PlainDword::new(0x3F0A3D71),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x26),
                        PlainDword::new(0x69),
                        PlainDword::new(0x32),
                        PlainDword::new(0x8D),
                    ],
                    engine_top: PlainDword::new(0x203A),
                    over_top: PlainDword::new(0xBB),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[4] = CarSpec {
                    name: PlainBytes::new(*b"04 Subaru Impreza       "),
                    drive_kind: PlainDword::new(1),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.2),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(1.85)],
                    whole_figures: [
                        PlainDword::new(0x83D),
                        PlainDword::new(0x83D),
                        PlainDword::new(0x35F),
                    ],
                    unused_4c: [
                        PlainDword::new(0x4CE),
                        PlainDword::new(0x402147AE),
                        PlainDword::new(0x3F0A3D71),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x31),
                        PlainDword::new(0x71),
                        PlainDword::new(0x3B),
                        PlainDword::new(0x93),
                    ],
                    engine_top: PlainDword::new(0x186A),
                    over_top: PlainDword::new(0xFF),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x3F19999A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F666666),
                        PlainDword::new(0x3F19999A),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.87),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[5] = CarSpec {
                    name: PlainBytes::new(*b"05 Peugeot 106          "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.15),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.91),
                        PlainFloat::new(0.97),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.25)],
                    whole_figures: [
                        PlainDword::new(0x736),
                        PlainDword::new(0x76D),
                        PlainDword::new(0x36A),
                    ],
                    unused_4c: [
                        PlainDword::new(0x37A),
                        PlainDword::new(0x4018F5C3),
                        PlainDword::new(0x3F028F5C),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x25),
                        PlainDword::new(0x5E),
                        PlainDword::new(0x2F),
                        PlainDword::new(0x83),
                    ],
                    engine_top: PlainDword::new(0x20D0),
                    over_top: PlainDword::new(0x9B),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[6] = CarSpec {
                    name: PlainBytes::new(*b"06 Citroen Maxo         "),
                    drive_kind: PlainDword::new(1),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.25)],
                    whole_figures: [
                        PlainDword::new(0x74F),
                        PlainDword::new(0x759),
                        PlainDword::new(0x37F),
                    ],
                    unused_4c: [
                        PlainDword::new(0x370),
                        PlainDword::new(0x401A3D71),
                        PlainDword::new(0x3F028F5C),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x32),
                        PlainDword::new(0x6E),
                        PlainDword::new(0x3C),
                        PlainDword::new(0x96),
                    ],
                    engine_top: PlainDword::new(0x1964),
                    over_top: PlainDword::new(0x113),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x3F19999A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F666666),
                        PlainDword::new(0x3F19999A),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.87),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[7] = CarSpec {
                    name: PlainBytes::new(*b"07 Mitsubishi Lancer    "),
                    drive_kind: PlainDword::new(1),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.92),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.99),
                        PlainFloat::new(1.02),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(1.95)],
                    whole_figures: [
                        PlainDword::new(0x81C),
                        PlainDword::new(0x8D6),
                        PlainDword::new(0x375),
                    ],
                    unused_4c: [
                        PlainDword::new(0x4CE),
                        PlainDword::new(0x4020A3D7),
                        PlainDword::new(0x3F147AE1),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x32),
                        PlainDword::new(0x73),
                        PlainDword::new(0x3C),
                        PlainDword::new(0x98),
                    ],
                    engine_top: PlainDword::new(0x1838),
                    over_top: PlainDword::new(0x10E),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x3F19999A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F666666),
                        PlainDword::new(0x3F19999A),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.87),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[8] = CarSpec {
                    name: PlainBytes::new(*b"08 VW Golf              "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.97),
                    side_grip_scale: PlainFloat::new(1.02),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.96),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.2)],
                    whole_figures: [
                        PlainDword::new(0x78E),
                        PlainDword::new(0x832),
                        PlainDword::new(0x375),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3D4),
                        PlainDword::new(0x40215810),
                        PlainDword::new(0x3F07AE14),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x25),
                        PlainDword::new(0x6A),
                        PlainDword::new(0x32),
                        PlainDword::new(0x8C),
                    ],
                    engine_top: PlainDword::new(0x2328),
                    over_top: PlainDword::new(0xB8),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[9] = CarSpec {
                    name: PlainBytes::new(*b"09 Nissan Almera        "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.92),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.91),
                        PlainFloat::new(0.95),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.3)],
                    whole_figures: [
                        PlainDword::new(0x7DA),
                        PlainDword::new(0x811),
                        PlainDword::new(0x380),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3C0),
                        PlainDword::new(0x40223D71),
                        PlainDword::new(0x3F0CCCCD),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x26),
                        PlainDword::new(0x68),
                        PlainDword::new(0x31),
                        PlainDword::new(0x8C),
                    ],
                    engine_top: PlainDword::new(0x2328),
                    over_top: PlainDword::new(0xB8),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[10] = CarSpec {
                    name: PlainBytes::new(*b"10 Proton Wira          "),
                    drive_kind: PlainDword::new(1),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.92),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.92),
                        PlainFloat::new(0.92),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(1.95)],
                    whole_figures: [
                        PlainDword::new(0x848),
                        PlainDword::new(0x8CB),
                        PlainDword::new(0x38B),
                    ],
                    unused_4c: [
                        PlainDword::new(0x4E7),
                        PlainDword::new(0x40200000),
                        PlainDword::new(0x3F1EB852),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x2F),
                        PlainDword::new(0x6D),
                        PlainDword::new(0x39),
                        PlainDword::new(0x8C),
                    ],
                    engine_top: PlainDword::new(0x1770),
                    over_top: PlainDword::new(0xEE),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x3F19999A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F666666),
                        PlainDword::new(0x3F19999A),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.87),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[11] = CarSpec {
                    name: PlainBytes::new(*b"11 Citroen Saxo         "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.1),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.92),
                        PlainFloat::new(0.99),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.15)],
                    whole_figures: [
                        PlainDword::new(0x74F),
                        PlainDword::new(0x759),
                        PlainDword::new(0x37F),
                    ],
                    unused_4c: [
                        PlainDword::new(0x370),
                        PlainDword::new(0x401A3D71),
                        PlainDword::new(0x3F028F5C),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x25),
                        PlainDword::new(0x63),
                        PlainDword::new(0x30),
                        PlainDword::new(0x86),
                    ],
                    engine_top: PlainDword::new(0x20D0),
                    over_top: PlainDword::new(0xA4),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[12] = CarSpec {
                    name: PlainBytes::new(*b"12 Honda Civic          "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.96),
                    side_grip_scale: PlainFloat::new(1.02),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.9),
                        PlainFloat::new(0.96),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.0)],
                    whole_figures: [
                        PlainDword::new(0x841),
                        PlainDword::new(0x7C7),
                        PlainDword::new(0x380),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3F2),
                        PlainDword::new(0x4030A3D7),
                        PlainDword::new(0x3F0CCCCD),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x26),
                        PlainDword::new(0x64),
                        PlainDword::new(0x32),
                        PlainDword::new(0x87),
                    ],
                    engine_top: PlainDword::new(0x24B8),
                    over_top: PlainDword::new(0xA8),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[13] = CarSpec {
                    name: PlainBytes::new(*b"13 Nissan Micra         "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.98),
                    side_grip_scale: PlainFloat::new(1.05),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.89),
                        PlainFloat::new(0.99),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.3)],
                    whole_figures: [
                        PlainDword::new(0x74E),
                        PlainDword::new(0x6C9),
                        PlainDword::new(0x306),
                    ],
                    unused_4c: [
                        PlainDword::new(0x366),
                        PlainDword::new(0x401CCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x23),
                        PlainDword::new(0x5D),
                        PlainDword::new(0x2D),
                        PlainDword::new(0x7F),
                    ],
                    engine_top: PlainDword::new(0x238C),
                    over_top: PlainDword::new(0x94),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[14] = CarSpec {
                    name: PlainBytes::new(*b"14 Peugeot 206          "),
                    drive_kind: PlainDword::new(1),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.1),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.98),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(1.85)],
                    whole_figures: [
                        PlainDword::new(0x84C),
                        PlainDword::new(0x79B),
                        PlainDword::new(0x36A),
                    ],
                    unused_4c: [
                        PlainDword::new(0x4CE),
                        PlainDword::new(0x401DF3B6),
                        PlainDword::new(0x3F0CCCCD),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x32),
                        PlainDword::new(0x72),
                        PlainDword::new(0x3B),
                        PlainDword::new(0x96),
                    ],
                    engine_top: PlainDword::new(0x1806),
                    over_top: PlainDword::new(0x105),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x3F19999A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F666666),
                        PlainDword::new(0x3F19999A),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.87),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[15] = CarSpec {
                    name: PlainBytes::new(*b"15 Ford Escort RS2000   "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.99),
                    side_grip_scale: PlainFloat::new(1.15),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.92),
                        PlainFloat::new(0.91),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.1)],
                    whole_figures: [
                        PlainDword::new(0x7BC),
                        PlainDword::new(0x84C),
                        PlainDword::new(0x35F),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3C0),
                        PlainDword::new(0x4021EB85),
                        PlainDword::new(0x3F0F5C29),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x24),
                        PlainDword::new(0x66),
                        PlainDword::new(0x2F),
                        PlainDword::new(0x88),
                    ],
                    engine_top: PlainDword::new(0x1FA4),
                    over_top: PlainDword::new(0xAE),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[16] = CarSpec {
                    name: PlainBytes::new(*b"16 Skoda Octavia        "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.9),
                    side_grip_scale: PlainFloat::new(1.02),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.93),
                        PlainFloat::new(0.9),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.3)],
                    whole_figures: [
                        PlainDword::new(0x848),
                        PlainDword::new(0x8F9),
                        PlainDword::new(0x367),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3D4),
                        PlainDword::new(0x4020C49C),
                        PlainDword::new(0x3F1EB852),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x26),
                        PlainDword::new(0x67),
                        PlainDword::new(0x30),
                        PlainDword::new(0x8B),
                    ],
                    engine_top: PlainDword::new(0x206C),
                    over_top: PlainDword::new(0xB1),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[17] = CarSpec {
                    name: PlainBytes::new(*b"17 Seat Ibiza           "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.98),
                    side_grip_scale: PlainFloat::new(1.1),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.94),
                        PlainFloat::new(0.95),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.35)],
                    whole_figures: [
                        PlainDword::new(0x79A),
                        PlainDword::new(0x6F8),
                        PlainDword::new(0x35D),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3C5),
                        PlainDword::new(0x401C3958),
                        PlainDword::new(0x3F0A3D71),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x25),
                        PlainDword::new(0x68),
                        PlainDword::new(0x31),
                        PlainDword::new(0x8B),
                    ],
                    engine_top: PlainDword::new(0x251C),
                    over_top: PlainDword::new(0xB5),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[18] = CarSpec {
                    name: PlainBytes::new(*b"18 Skoda Felicia        "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.97),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.88),
                        PlainFloat::new(0.96),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.25)],
                    whole_figures: [
                        PlainDword::new(0x7B9),
                        PlainDword::new(0x735),
                        PlainDword::new(0x349),
                    ],
                    unused_4c: [
                        PlainDword::new(0x352),
                        PlainDword::new(0x401CCCCD),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x23),
                        PlainDword::new(0x5D),
                        PlainDword::new(0x2D),
                        PlainDword::new(0x7F),
                    ],
                    engine_top: PlainDword::new(0x1F40),
                    over_top: PlainDword::new(0x98),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[19] = CarSpec {
                    name: PlainBytes::new(*b"19 Proton Compact       "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.95),
                    side_grip_scale: PlainFloat::new(1.05),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.9),
                        PlainFloat::new(0.95),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.25)],
                    whole_figures: [
                        PlainDword::new(0x800),
                        PlainDword::new(0x767),
                        PlainDword::new(0x35D),
                    ],
                    unused_4c: [
                        PlainDword::new(0x375),
                        PlainDword::new(0x401C28F6),
                        PlainDword::new(0x3F0CCCCD),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x24),
                        PlainDword::new(0x5B),
                        PlainDword::new(0x2C),
                        PlainDword::new(0x7F),
                    ],
                    engine_top: PlainDword::new(0x2134),
                    over_top: PlainDword::new(0x92),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[20] = CarSpec {
                    name: PlainBytes::new(*b"20 Ford Maxi            "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.97),
                    side_grip_scale: PlainFloat::new(1.02),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.93),
                        PlainFloat::new(0.94),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.15)],
                    whole_figures: [
                        PlainDword::new(0x7B4),
                        PlainDword::new(0x868),
                        PlainDword::new(0x392),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3C0),
                        PlainDword::new(0x4021EB85),
                        PlainDword::new(0x3F0CCCCD),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x25),
                        PlainDword::new(0x67),
                        PlainDword::new(0x31),
                        PlainDword::new(0x8A),
                    ],
                    engine_top: PlainDword::new(0x2198),
                    over_top: PlainDword::new(0xB2),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[21] = CarSpec {
                    name: PlainBytes::new(*b"21 Ford Puma            "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.98),
                    side_grip_scale: PlainFloat::new(1.1),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.92),
                        PlainFloat::new(0.97),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.4)],
                    whole_figures: [
                        PlainDword::new(0x7C0),
                        PlainDword::new(0x793),
                        PlainDword::new(0x370),
                    ],
                    unused_4c: [
                        PlainDword::new(0x316),
                        PlainDword::new(0x401C28F6),
                        PlainDword::new(0x3F07AE14),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x28),
                        PlainDword::new(0x5A),
                        PlainDword::new(0x32),
                        PlainDword::new(0x7E),
                    ],
                    engine_top: PlainDword::new(0x2134),
                    over_top: PlainDword::new(0x96),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[22] = CarSpec {
                    name: PlainBytes::new(*b"22 Mitsubishi EVO 4     "),
                    drive_kind: PlainDword::new(1),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(0.92),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(0.94),
                        PlainFloat::new(0.95),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(1.9)],
                    whole_figures: [
                        PlainDword::new(0x82F),
                        PlainDword::new(0x8C0),
                        PlainDword::new(0x338),
                    ],
                    unused_4c: [
                        PlainDword::new(0x4CE),
                        PlainDword::new(0x4020A3D7),
                        PlainDword::new(0x3F147AE1),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x30),
                        PlainDword::new(0x6F),
                        PlainDword::new(0x3A),
                        PlainDword::new(0x90),
                    ],
                    engine_top: PlainDword::new(0x1838),
                    over_top: PlainDword::new(0xF9),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x3F19999A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F666666),
                        PlainDword::new(0x3F19999A),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.87),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[23] = CarSpec {
                    name: PlainBytes::new(*b"23 MF Supercar          "),
                    drive_kind: PlainDword::new(1),
                    gear_count: PlainDword::new(7),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.2)],
                    whole_figures: [
                        PlainDword::new(0x7F0),
                        PlainDword::new(0x81C),
                        PlainDword::new(0x375),
                    ],
                    unused_4c: [
                        PlainDword::new(0x4E2),
                        PlainDword::new(0x401F0A3D),
                        PlainDword::new(0x3F0CCCCD),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x37),
                        PlainDword::new(0x82),
                        PlainDword::new(0x41),
                        PlainDword::new(0xAA),
                    ],
                    engine_top: PlainDword::new(0x1B58),
                    over_top: PlainDword::new(0x140),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x3F19999A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F666666),
                        PlainDword::new(0x3F19999A),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.87),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[24] = CarSpec {
                    name: PlainBytes::new(*b"24 MF Boyota            "),
                    drive_kind: PlainDword::new(1),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.2)],
                    whole_figures: [
                        PlainDword::new(0x7D0),
                        PlainDword::new(0x7D0),
                        PlainDword::new(0x3E8),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3E8),
                        PlainDword::new(0x4028F5C3),
                        PlainDword::new(0x3E800000),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x32),
                        PlainDword::new(0x7D),
                        PlainDword::new(0x3C),
                        PlainDword::new(0xA0),
                    ],
                    engine_top: PlainDword::new(0x1A90),
                    over_top: PlainDword::new(0x12C),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x3F19999A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F666666),
                        PlainDword::new(0x3F19999A),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[25] = CarSpec {
                    name: PlainBytes::new(*b"25 Pug 309              "),
                    gear_count: PlainDword::new(5),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.2)],
                    whole_figures: [
                        PlainDword::new(0x7D0),
                        PlainDword::new(0x7D0),
                        PlainDword::new(0x3E8),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3E8),
                        PlainDword::new(0x4028F5C3),
                        PlainDword::new(0x3E800000),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x23),
                        PlainDword::new(0x5A),
                        PlainDword::new(0x2D),
                        PlainDword::new(0x7D),
                    ],
                    engine_top: PlainDword::new(0x1D4C),
                    over_top: PlainDword::new(0x96),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[26] = CarSpec {
                    name: PlainBytes::new(*b"26 Skip                 "),
                    gear_count: PlainDword::new(5),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.2)],
                    whole_figures: [
                        PlainDword::new(0x7D0),
                        PlainDword::new(0x708),
                        PlainDword::new(0x3E8),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3E8),
                        PlainDword::new(0x4028F5C3),
                        PlainDword::new(0x3E800000),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x1E),
                        PlainDword::new(0x46),
                        PlainDword::new(0x28),
                        PlainDword::new(0x5A),
                    ],
                    engine_top: PlainDword::new(0x1964),
                    over_top: PlainDword::new(0x50),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F000000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[27] = CarSpec {
                    name: PlainBytes::new(*b"27 Potato               "),
                    drive_kind: PlainDword::new(1),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.2)],
                    whole_figures: [
                        PlainDword::new(0x3E8),
                        PlainDword::new(0x3E8),
                        PlainDword::new(0x3E8),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3E8),
                        PlainDword::new(0x4028F5C3),
                        PlainDword::new(0x3E800000),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x28),
                        PlainDword::new(0x6E),
                        PlainDword::new(0x30),
                        PlainDword::new(0x8C),
                    ],
                    engine_top: PlainDword::new(0x157C),
                    over_top: PlainDword::new(0xF0),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F99999A),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[28] = CarSpec {
                    name: PlainBytes::new(*b"28 Cow                  "),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.2)],
                    whole_figures: [
                        PlainDword::new(0x3E8),
                        PlainDword::new(0x546),
                        PlainDword::new(0x28A),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3E8),
                        PlainDword::new(0x4028F5C3),
                        PlainDword::new(0x3E800000),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x23),
                        PlainDword::new(0x5A),
                        PlainDword::new(0x2D),
                        PlainDword::new(0x78),
                    ],
                    engine_top: PlainDword::new(0x1F40),
                    over_top: PlainDword::new(0x8C),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[29] = CarSpec {
                    name: PlainBytes::new(*b"29 Sheep                "),
                    gear_count: PlainDword::new(5),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.2)],
                    whole_figures: [
                        PlainDword::new(0x320),
                        PlainDword::new(0x320),
                        PlainDword::new(0x2BC),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3E8),
                        PlainDword::new(0x4028F5C3),
                        PlainDword::new(0x3E800000),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x23),
                        PlainDword::new(0x55),
                        PlainDword::new(0x28),
                        PlainDword::new(0x6E),
                    ],
                    engine_top: PlainDword::new(0x1D4C),
                    over_top: PlainDword::new(0x7D),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[30] = CarSpec {
                    name: PlainBytes::new(*b"30 RC Ibiza             "),
                    gear_count: PlainDword::new(5),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.2)],
                    whole_figures: [
                        PlainDword::new(0xA8C),
                        PlainDword::new(0xA8C),
                        PlainDword::new(0x3E8),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3E8),
                        PlainDword::new(0x4028F5C3),
                        PlainDword::new(0x3E800000),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x28),
                        PlainDword::new(0x64),
                        PlainDword::new(0x2D),
                        PlainDword::new(0x82),
                    ],
                    engine_top: PlainDword::new(0x2134),
                    over_top: PlainDword::new(0xA0),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x3F19999A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F666666),
                        PlainDword::new(0x3F19999A),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F000000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[31] = CarSpec {
                    name: PlainBytes::new(*b"31 Volvo Estate         "),
                    gear_count: PlainDword::new(5),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.2)],
                    whole_figures: [
                        PlainDword::new(0xA8C),
                        PlainDword::new(0xA8C),
                        PlainDword::new(0x3E8),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3E8),
                        PlainDword::new(0x4028F5C3),
                        PlainDword::new(0x3E800000),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x28),
                        PlainDword::new(0x64),
                        PlainDword::new(0x2D),
                        PlainDword::new(0x82),
                    ],
                    engine_top: PlainDword::new(0x2134),
                    over_top: PlainDword::new(0xA0),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x3F19999A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F666666),
                        PlainDword::new(0x3F19999A),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F000000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[32] = CarSpec {
                    name: PlainBytes::new(*b"32 Local Shop           "),
                    gear_count: PlainDword::new(5),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.0),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.2)],
                    whole_figures: [
                        PlainDword::new(0x640),
                        PlainDword::new(0x708),
                        PlainDword::new(0x384),
                    ],
                    unused_4c: [
                        PlainDword::new(0x3E8),
                        PlainDword::new(0x4028F5C3),
                        PlainDword::new(0x3E800000),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x1E),
                        PlainDword::new(0x4B),
                        PlainDword::new(0x28),
                        PlainDword::new(0x64),
                    ],
                    engine_top: PlainDword::new(0x1964),
                    over_top: PlainDword::new(0x78),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3EE147AE),
                        PlainDword::new(0x3F051EB8),
                        PlainDword::new(0x3F23D70A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F6B851F),
                        PlainDword::new(0x3F7AE148),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F7AE148),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3FC00000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a[33] = CarSpec {
                    name: PlainBytes::new(*b"33 Renault Sport Clio   "),
                    drive_kind: PlainDword::new(1),
                    gear_count: PlainDword::new(6),
                    grip_scale: PlainFloat::new(1.0),
                    side_grip_scale: PlainFloat::new(1.1),
                    surface: [
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                        PlainFloat::new(1.0),
                    ],
                    weights: [PlainFloat::new(1.0), PlainFloat::new(2.2)],
                    whole_figures: [
                        PlainDword::new(0x76C),
                        PlainDword::new(0x76C),
                        PlainDword::new(0x36B),
                    ],
                    unused_4c: [
                        PlainDword::new(0x44C),
                        PlainDword::new(0x401C28F6),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x40C00000),
                    ],
                    scratch_value: PlainDword::new(0x3F000000),
                    downforce: PlainDword::new(0x3DCCCCCD),
                    figures: [
                        PlainDword::new(0x32),
                        PlainDword::new(0x7D),
                        PlainDword::new(0x3C),
                        PlainDword::new(0xA0),
                    ],
                    engine_top: PlainDword::new(0x2134),
                    over_top: PlainDword::new(0x12C),
                    torque_curve: [
                        PlainDword::new(0x3ECCCCCD),
                        PlainDword::new(0x3F000000),
                        PlainDword::new(0x3F19999A),
                        PlainDword::new(0x3F4CCCCD),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F800000),
                        PlainDword::new(0x3F666666),
                        PlainDword::new(0x3F19999A),
                    ],
                    ride_height_handling_scale: PlainDword::new(0x3F800000),
                    shift_points: [
                        PlainFloat::new(0.95),
                        PlainFloat::new(0.6),
                        PlainFloat::new(0.7),
                    ],
                    ..ZeroCells::ZERO
                };
                a
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cars_start_as_the_games() {
        let physics = &race().physics;
        let title = |k: usize| {
            (0..9)
                .map(|i| physics.car_specs().cars[k].name.get(i))
                .collect::<Vec<u8>>()
        };
        assert_eq!(title(0), b"00 renaul");
        assert_eq!(title(33), b"33 Renaul");
        assert_eq!(physics.car_specs().cars[0].engine_top.get(), 8500);
    }
}
