//! The ground probe's mark: tells the ground search how it was called. A plain, `Copy` record:
//! held by the physics owner (`race::owner_physics::Physics::ground_probe`) and copied into the
//! drawing parts' reads when a drawing scope opens (the shadow's search reads the mark). Names are
//! readings unless taken from the game's own.

#[derive(Clone, Copy)]
pub struct GroundProbe {
    /// 0x40 or 0x80 by the caller of the ground search (`physics::count_ground_probe`,
    /// `physics::move_slot`, `physics::probe_ground`, `race::car_starts`).
    pub settle_mark: u32,
}

/// The piece, with the image's first values.
impl GroundProbe {
    pub const fn new() -> Self {
        Self { settle_mark: 128 }
    }
}
