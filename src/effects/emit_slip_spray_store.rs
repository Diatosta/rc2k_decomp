//! The slip spray's set.

use crate::memory::session::{PlainDword, PlainFloat};

/// Kind 5's emitter variables.
#[derive(rc2k_derive::Cells)]
pub struct SlipSpraySet {
    /// The place's random
    pub place_random: PlainDword,
    /// The motion's x factor
    pub motion_x: PlainFloat,
    /// The motion's y factor
    pub motion_y: PlainFloat,
    /// Added to the motion's y
    pub lift: PlainFloat,
    /// The motion's z factor
    pub motion_z: PlainFloat,
    /// The spread `jitter_point` takes
    pub spread: PlainFloat,
    /// The third row's factors (z, y, x)
    pub roll_k: [PlainFloat; 3],
    /// The spin's factor
    pub spin: PlainFloat,
}

impl SlipSpraySet {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SlipSpraySet {
            place_random: PlainDword::new(0x0),
            motion_x: PlainFloat::new(10.0),
            motion_y: PlainFloat::new(10.0),
            lift: PlainFloat::new(15.0),
            motion_z: PlainFloat::new(10.0),
            spread: PlainFloat::new(110.0),
            roll_k: [
                PlainFloat::new(-12.0),
                PlainFloat::new(-12.0),
                PlainFloat::new(-12.0),
            ],
            spin: PlainFloat::new(-64.0),
        }
    }
}
