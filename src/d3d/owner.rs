//! Module d3d's owner (step 11, owned state): the 3D cards' store in one struct, reached
//! through `d3d()` by the code that does not take it as a parameter.

use crate::d3d::cards_store::Cards3d;

/// The d3d module's state.
pub struct D3d {
    /// The 3D cards' variables (current texture, render states, modes).
    /// One copy per drawing thread (`workers`): the states and texture a thread's fans are
    /// noted with.
    pub cards: crate::workers::PerWorker<Cards3d>,
}

impl D3d {
    pub const fn new() -> Self {
        D3d {
            cards: crate::workers::PerWorker::new(
                [const { Cards3d::new() }; crate::workers::COPIES],
            ),
        }
    }
}

/// The one instance. Reached through `d3d()`; `moved` registers its stores' places.
pub static D3D: D3d = D3d::new();

/// The d3d module's state, for a caller that does not take it as a parameter yet.
pub fn d3d() -> &'static D3d {
    &D3D
}
