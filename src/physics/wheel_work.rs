//! The drive's working figures of the physics step: the wheels' slips, the two axles' drive
//! forces and slips, the drive's slip and the force sums `physics::sum_forces` makes. Plain
//! fields (a float held as its bits, as the recorded vectors feed them), borrowed for one
//! routine at a time (`Physics::wheel_work`).

/// The drive's working figures; each is written by the step before it is read.
#[derive(Clone, Default)]
pub struct WheelWork {
    /// |front / rear wheel speed - the car's| (`physics::drive_wheels_step`).
    pub wheel_slips: [u32; 2],
    /// The two axles' drive forces.
    pub drive_forces: [u32; 2],
    /// The drive's slip and its eased value.
    pub drive_slip: [u32; 2],
    /// The two axles' slips.
    pub axle_slips: [u32; 2],
    /// `physics::sum_forces`' five sums.
    pub force_sums: [u32; 5],
}

impl WheelWork {
    /// Every figure 0.
    pub const fn new() -> Self {
        WheelWork {
            wheel_slips: [0; 2],
            drive_forces: [0; 2],
            drive_slip: [0; 2],
            axle_slips: [0; 2],
            force_sums: [0; 5],
        }
    }
}
