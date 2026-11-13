//! The gearbox figures and the test drive's results, 0x5CB958 .. 0x5CC198: the shift points the
//! car file sets (`physics::load_car_specs`), the figures beside them, and the test drive's
//! marks and samples (`physics::test_drive_run`, `physics::test_drive`).
//! Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::{PlainDword, PlainFloat, PlainInt};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct GearFigures {
    /// +0x0 (0x5CB958): `flt_5CB958`: the engine figure above which the gearbox shifts up (`physics::auto_gearbox`, `physics::manual_gearbox`, `physics::pick_start_gear`).
    pub shift_up_point: PlainFloat,
    /// +0x4 (0x5CB95C): `flt_5CB95C`: the figure below which it shifts down (`physics::auto_gearbox`, `physics::manual_gearbox`).
    pub shift_down_point: PlainFloat,
    /// +0x8 (0x5CB960): `flt_5CB960`: the same with the clutch in (`physics::auto_gearbox`, `physics::manual_gearbox`).
    pub shift_down_clutch_point: PlainFloat,
    /// +0xc (0x5CB964): `flt_5CB964`: the figure the ratio check compares with (`physics::auto_gearbox`, `physics::manual_gearbox`).
    pub shift_ratio_figure: PlainFloat,
    /// +0x10 (0x5CB968): `flt_5CB968`: multiplied into the drive torque (`physics::drive_force`; the car file sets it, `physics::load_car_specs`).
    pub drive_torque_figure: PlainFloat,
    /// +0x14 (0x5CB96C): `flt_5CB96C` .. `flt_5CB978`: the four figures the car file sets, each held to at least 8.0 (`physics::load_car_specs`, `physics::make_gear_ratios`).
    pub figures: [PlainFloat; 4],
    /// +0x24 (0x5CB97C): `dword_5CB97C` .. `dword_5CB984`: the test drive's step marks (`physics::test_drive_run`).
    pub test_marks: [PlainDword; 3],
    /// +0x30 (0x5CB988): `flt_5CB988`, `flt_5CB98C`: the speeds the marks wait for (`physics::test_drive_run`).
    pub test_limits: [PlainFloat; 2],
    /// +0x38 (0x5CB990): `flt_5CB990`: the test drive's speed samples, one in twenty steps (`physics::test_drive_run`).
    pub test_speeds: [PlainFloat; 256],
    /// +0x438 (0x5CBD90): `dword_5CBD90`: the gear at each sample (`physics::test_drive_run`).
    pub test_gears: [PlainInt; 256],
    /// +0x838 (0x5CC190): `dword_5CC190`: the engine power factor the test drive sets (`physics::test_drive`).
    pub test_power_factor: PlainFloat,
    /// +0x83c (0x5CC194): a busy flag the game sets around its copy of the "demof" file (guards the copy hook against re-entry); no port code names it.
    pub demo_copy_busy: PlainDword,
}

impl GearFigures {
    /// The gear figures as the game starts: the test drive's speed limits and power factor.
    pub const fn new() -> Self {
        GearFigures {
            test_limits: [PlainFloat::new(26.822428), PlainFloat::new(44.70405)],
            test_power_factor: PlainFloat::new(1.0),
            ..ZeroCells::ZERO
        }
    }
}
