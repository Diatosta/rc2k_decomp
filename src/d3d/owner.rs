//! The 3D cards' store in one struct, held by the graphics owner (`Gfx::d3d`).

use crate::d3d::cards_store::Cards3d;
use crate::memory::session::GameThread;

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
