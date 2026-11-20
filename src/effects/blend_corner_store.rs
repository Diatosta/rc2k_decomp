//! The corner blend's work.

use crate::memory::session::PlainFloat;

/// The corner blend's factor.
#[derive(rc2k_derive::Cells)]
pub struct BlendWork {
    /// The blend factor (`blend_corner`'s)
    pub factor: PlainFloat,
}

impl BlendWork {
    /// BlendWork's first values.
    pub const fn new() -> Self {
        Self {
            factor: PlainFloat::new(0.0),
        }
    }
}
