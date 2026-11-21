//! The gearbox figures and the test drive's results: the shift points the car file sets
//! (`physics::load_car_specs`), the figures beside them, and the test drive's marks and samples
//! (`physics::acceleration_run`, `physics::acceleration_figures`). Names are readings unless taken from the
//! game's own.

use crate::memory::session::{PlainDword, PlainFloat, PlainInt};

/// The figures, held by the game's thread (`race::owner_physics::Physics::gear`).
pub struct GearFigures {
    /// The engine figure above which the gearbox shifts up (`physics::auto_gearbox`,
    /// `physics::manual_gearbox`, `physics::pick_start_gear`).
    pub shift_up_point: PlainFloat,
    /// The figure below which it shifts down (`physics::auto_gearbox`,
    /// `physics::manual_gearbox`).
    pub shift_down_point: PlainFloat,
    /// The same with the clutch in (`physics::auto_gearbox`,
    /// `physics::manual_gearbox`).
    pub shift_down_clutch_point: PlainFloat,
    /// The figure the ratio check compares with (`physics::auto_gearbox`,
    /// `physics::manual_gearbox`).
    pub shift_ratio_figure: PlainFloat,
    /// Multiplied into the drive torque (`physics::drive_force`; the car file sets
    /// it, `physics::load_car_specs`).
    pub drive_torque_figure: PlainFloat,
    /// The four figures the car file sets, each held to at least 8.0
    /// (`physics::load_car_specs`, `physics::make_gear_ratios`).
    pub figures: [PlainFloat; 4],
    /// The test drive's step marks (`physics::acceleration_run`).
    pub test_marks: [PlainDword; 3],
    /// The speeds the marks wait for (`physics::acceleration_run`).
    pub test_limits: [PlainFloat; 2],
    /// The test drive's speed samples, one in twenty steps
    /// (`physics::acceleration_run`).
    pub test_speeds: [PlainFloat; 256],
    /// The gear at each sample (`physics::acceleration_run`).
    pub test_gears: [PlainInt; 256],
    /// The engine power factor the test drive sets (`physics::acceleration_figures`).
    pub test_power_factor: PlainFloat,
}

impl GearFigures {
    /// The gear figures at the start: the test drive's speed limits and power factor.
    pub const fn new() -> Self {
        GearFigures {
            shift_up_point: PlainFloat::new(0.0),
            shift_down_point: PlainFloat::new(0.0),
            shift_down_clutch_point: PlainFloat::new(0.0),
            shift_ratio_figure: PlainFloat::new(0.0),
            drive_torque_figure: PlainFloat::new(0.0),
            figures: [const { PlainFloat::new(0.0) }; 4],
            test_marks: [const { PlainDword::new(0) }; 3],
            test_limits: [PlainFloat::new(26.822428), PlainFloat::new(44.70405)],
            test_speeds: [const { PlainFloat::new(0.0) }; 256],
            test_gears: [const { PlainInt::new(0) }; 256],
            test_power_factor: PlainFloat::new(1.0),
        }
    }
}
