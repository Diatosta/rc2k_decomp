//! The wheel dust cloud's set.

use crate::memory::session::{PlainDword, PlainFloat};

/// Kind 6's emitter variables (`emit_wheel_dust_cloud`).
#[derive(rc2k_derive::Cells)]
pub struct WheelFadeSet {
    /// The rate's factors
    pub rate_k: [PlainFloat; 4],
    /// The side's random
    pub side_random: PlainDword,
    /// The place's drop in y
    pub lift: PlainFloat,
    /// Per axis (x, y, z) the scales of the three products (the wheel's `local_velocity_x`,
    /// `local_velocity_z`, `driven_speed`)
    pub motion_k: [PlainFloat; 9],
    /// The spin's random
    pub spin_random: PlainDword,
    /// The size's factors
    pub size_k: [PlainFloat; 2],
}

impl WheelFadeSet {
    /// The values the game starts with.
    pub const fn new() -> Self {
        WheelFadeSet {
            rate_k: [
                PlainFloat::new(65.336),
                PlainFloat::new(0.0002),
                PlainFloat::new(1.0),
                PlainFloat::new(196608.0),
            ],
            side_random: PlainDword::new(0x0),
            lift: PlainFloat::new(65536.0),
            motion_k: [
                PlainFloat::new(65.536),
                PlainFloat::new(-49.152),
                PlainFloat::new(26.152),
                PlainFloat::new(65.536),
                PlainFloat::new(-49.152),
                PlainFloat::new(26.152),
                PlainFloat::new(65.536),
                PlainFloat::new(-49.152),
                PlainFloat::new(26.152),
            ],
            spin_random: PlainDword::new(0x0),
            size_k: [PlainFloat::new(0.3), PlainFloat::new(8192.0)],
        }
    }
}
