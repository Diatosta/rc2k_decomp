//! The arcade game's state: the level and stage being played, the retry count and the levels
//! won. One static, owned by the menus.

use crate::session::{PlainBytes, PlainDword};

/// The arcade game's state.
#[derive(rc2k_derive::Cells)]
pub struct Arcade {
    /// 1 when the front end's scene was let go for a race (`menu::reload_front_end` sets it up
    /// again).
    pub scene_let_go: PlainDword,
    /// The arcade level being played.
    pub level: PlainDword,
    /// The arcade stage being played.
    pub stage: PlainDword,
    /// 1 after a lost level (the retry screens follow).
    pub retry: PlainDword,
    /// The places the arcade game ranks.
    pub places: PlainDword,
    /// One byte a level: 1 when the level is open (the options file keeps the first six).
    pub levels_won: PlainBytes<8>,
}

impl Arcade {
    pub const fn new() -> Self {
        Arcade {
            scene_let_go: PlainDword::new(0x0),
            level: PlainDword::new(0x0),
            stage: PlainDword::new(0x1),
            retry: PlainDword::new(0x0),
            places: PlainDword::new(0x64),
            levels_won: PlainBytes::new([0; 8]),
        }
    }

    /// The level and the stage being played.
    pub fn position(&self) -> (u32, u32) {
        (self.level.get(), self.stage.get())
    }

    /// Starts `level` at its first stage.
    pub fn start_level(&self, level: u32) {
        self.level.set(level);
        self.stage.set(0);
    }

    /// The opened byte of level `index` (0 based; the level after `index` is opened by winning
    /// it): 1 when open.
    pub fn opened(&self, index: usize) -> u8 {
        self.levels_won.get(index)
    }

    /// Marks level `index` (0 based) open.
    pub fn open(&self, index: usize) {
        self.levels_won.set(index, 1);
    }
}
