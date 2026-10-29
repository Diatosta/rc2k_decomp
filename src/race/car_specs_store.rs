//! The cars' spec records, 0x5C790C .. 0x5C906C: 34 records of 0xB0 bytes, one per car kind
//! (`g_carSpecs`, IDA struct `CarSpec`: "00 renault maxi megane" .. "33 Renault Sport Clio"), read by
//! `physics::load_car_specs` (the work car's handling figures) and `race::race_shell` (the
//! start values). `g_lightRecords` follows.
//! Names are readings unless IDA's.

use crate::session::{Bytes, Dword, Float};

/// One car kind's figures (IDA struct `CarSpec`).
#[repr(C)]
pub struct CarSpec {
    /// +0x0: the car's title, padded with spaces, e.g. "00 renault maxi megane"
    /// (drawn by `sub_4426D4`).
    pub name: Bytes<24>,
    /// +0x18: the drive kind, checked 0 ..= 3 (`physics::load_car_specs`); also the start
    /// value (`race::race_shell`).
    pub drive_kind: Dword,
    /// +0x1C: the gear count by its sign, held to 3 ..= 7 (`physics::load_car_specs`).
    pub gear_count: Dword,
    /// +0x20: copied to `dword_614C14`.
    pub copy_20: Dword,
    /// +0x24: copied to the physics `grip_scale` (a reading).
    pub grip_scale: Dword,
    /// +0x28: copied to the physics `side_grip_scale` (a reading).
    pub side_grip_scale: Dword,
    /// +0x2C .. +0x34: copied to the physics `spec_surface` 0, 1, 2 (+0x2C, +0x30, +0x34).
    pub surface: [Dword; 3],
    /// +0x38, +0x3C: two figures times `flt_613FC8` into `dword_614BAC`, `dword_614BB0`.
    pub weights: [Float; 2],
    /// +0x40: three whole figures x 65536 / `flt_5C5D18` into `dword_614BC0` .. (kinds
    /// from 0x17 on; the others take the game's constants).
    pub whole_figures: [Dword; 3],
    /// +0x4C: not read by the game's code.
    pub unknown_4c: [Dword; 4],
    /// +0x5C: into `dword_62400C` (`scene::draw_store`'s `scratch`).
    pub scratch_value: Dword,
    /// +0x60: into the physics `downforce_spec`.
    pub downforce: Dword,
    /// +0x64: four whole figures as floats into `flt_5CB96C` .. (each at least 8.0).
    pub figures: [Dword; 4],
    /// +0x74: the engine's top, a whole number (8500 for the first); also the start
    /// value's limit (`dword_5C7980`, `race::race_shell`).
    pub engine_top: Dword,
    /// +0x78: a whole figure divided by the top into `flt_5CB968`.
    pub over_top: Dword,
    /// +0x7C: the torque curve, copied to the physics `spec_torque`.
    pub torque_curve: [Dword; 9],
    /// +0xA0: into the physics `spec_a0`.
    pub spec_a0: Dword,
    /// +0xA4: the shift points (`flt_5CB958` ..; the first x `flt_613E88`, the others +
    /// `flt_5C5DDC`).
    pub shift_points: [Float; 3],
}

const _: () = assert!(core::mem::size_of::<CarSpec>() == 0xB0);

/// The car kinds' records.
#[repr(C)]
pub struct CarSpecs {
    pub cars: [CarSpec; 34],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.
unsafe impl crate::pointers::Cells for CarSpecs {}

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static CAR_SPECS: CarSpecs =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x5C790C, the first byte of the piece.
pub const START: usize = 0x5C790C;
/// IDA 0x5C906C, the end of the piece (`g_lightRecords`).
pub const END: usize = 0x5C906C;

const _: () = assert!(END - START == core::mem::size_of::<CarSpecs>());

/// Where IDA 0x5C790C is now (its own static).
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&CAR_SPECS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&CAR_SPECS, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(CarSpec, drive_kind), 0x18);
        assert_eq!(offset_of!(CarSpec, weights), 0x38);
        assert_eq!(offset_of!(CarSpec, scratch_value), 0x5C);
        assert_eq!(offset_of!(CarSpec, engine_top), 0x74);
        assert_eq!(offset_of!(CarSpec, torque_curve), 0x7C);
        assert_eq!(offset_of!(CarSpec, shift_points), 0xA4);
        let title = |k: usize| {
            (0..9)
                .map(|i| CAR_SPECS.cars[k].name.get(i))
                .collect::<Vec<u8>>()
        };
        assert_eq!(title(0), b"00 renaul");
        assert_eq!(title(33), b"33 Renaul");
        assert_eq!(CAR_SPECS.cars[0].engine_top.get(), 8500);
    }
}
