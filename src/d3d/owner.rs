//! Module d3d's owner: the 3D cards' store in one struct, reached through `d3d()` by the code
//! that does not take it as a parameter.

use crate::d3d::cards_store::Cards3d;
use crate::session::GameThread;

/// The d3d module's state.
pub struct D3d {
    /// The 3D cards' variables (the settings, the scene flag, the modes): the game's thread
    /// alone reaches them. The render states and the texture a part's fans are noted with are
    /// each part's own (`d3d::render_cache`).
    pub cards: GameThread<Cards3d>,
}

impl D3d {
    pub const fn new() -> Self {
        D3d {
            cards: GameThread::new(Cards3d::new()),
        }
    }
}

/// The one instance. Reached through `d3d()`; `moved` registers its stores' places.
pub static D3D: D3d = D3d::new();

/// The d3d module's state, for a caller that does not take it as a parameter yet.
pub fn d3d() -> &'static D3d {
    &D3D
}
