//! The gearbox figures and the test drive's results, 0x5CB958 .. 0x5CC198: the shift points the
//! car file sets (`physics::load_car_specs`), the figures beside them, and the test drive's
//! marks and samples (`physics::test_drive_run`, `physics::test_drive`).
//! Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
pub struct GearFigures {
    /// +0x0 (0x5CB958): `flt_5CB958`: the engine figure above which the gearbox shifts up (`physics::auto_gearbox`, `physics::manual_gearbox`, `physics::pick_start_gear`).
    pub shift_up_point: Float,
    /// +0x4 (0x5CB95C): `flt_5CB95C`: the figure below which it shifts down (`physics::auto_gearbox`, `physics::manual_gearbox`).
    pub shift_down_point: Float,
    /// +0x8 (0x5CB960): `flt_5CB960`: the same with the clutch in (`physics::auto_gearbox`, `physics::manual_gearbox`).
    pub shift_down_clutch_point: Float,
    /// +0xc (0x5CB964): `flt_5CB964`: the figure the ratio check compares with (`physics::auto_gearbox`, `physics::manual_gearbox`).
    pub shift_ratio_figure: Float,
    /// +0x10 (0x5CB968): `flt_5CB968`: multiplied into the drive torque (`physics::drive_force`; the car file sets it, `physics::load_car_specs`).
    pub drive_torque_figure: Float,
    /// +0x14 (0x5CB96C): `flt_5CB96C` .. `flt_5CB978`: the four figures the car file sets, each held to at least 8.0 (`physics::load_car_specs`, `physics::make_gear_ratios`).
    pub figures: [Float; 4],
    /// +0x24 (0x5CB97C): `dword_5CB97C` .. `dword_5CB984`: the test drive's step marks (`physics::test_drive_run`).
    pub test_marks: [Dword; 3],
    /// +0x30 (0x5CB988): `flt_5CB988`, `flt_5CB98C`: the speeds the marks wait for (`physics::test_drive_run`).
    pub test_limits: [Float; 2],
    /// +0x38 (0x5CB990): `flt_5CB990`: the test drive's speed samples, one in twenty steps (`physics::test_drive_run`).
    pub test_speeds: [Float; 256],
    /// +0x438 (0x5CBD90): `dword_5CBD90`: the gear at each sample (`physics::test_drive_run`).
    pub test_gears: [Dword; 256],
    /// +0x838 (0x5CC190): `dword_5CC190`: the engine power factor the test drive sets (`physics::test_drive`).
    pub test_power_factor: Float,
    /// +0x83c (0x5CC194): not named.
    pub unknown_83c: Dword,
}

/// The piece: a field of `race::groups_store::LIGHTS_AND_GEAR` since 2026-10-29 (it left the race
/// block with its group, phase 13).
pub struct InGroup;

impl core::ops::Deref for InGroup {
    type Target = GearFigures;
    fn deref(&self) -> &GearFigures {
        &crate::race::groups_store::LIGHTS_AND_GEAR.gear
    }
}

pub static GEAR: InGroup = InGroup;

/// IDA 0x5CB958, the first byte of the piece.
pub const START: usize = 0x5CB958;
/// IDA 0x5CC198, the end of the piece.
pub const END: usize = 0x5CC198;

const _: () = assert!(END - START == core::mem::size_of::<GearFigures>());

/// Where IDA 0x5CB958 is now (its group's static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &crate::race::groups_store::LIGHTS_AND_GEAR.gear,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(GearFigures, shift_up_point), 0x5CB958 - START);
        assert_eq!(offset_of!(GearFigures, figures), 0x5CB96C - START);
        assert_eq!(offset_of!(GearFigures, test_power_factor), 0x5CC190 - START);
    }
}
