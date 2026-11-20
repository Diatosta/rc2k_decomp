//! The arcade game's state: the level and stage being played, the retry count and the levels
//! won. One static, owned by the menus.

use crate::memory::session::GameThread;
use std::cell::{RefCell, RefMut};

/// The arcade game's state. Plain fields behind one `RefCell`: the game's thread alone reads
/// and writes them, each borrow lasting one statement.
pub struct Arcade(GameThread<RefCell<ArcadeValues>>);

/// The arcade game's values.
#[derive(Clone, Copy)]
pub struct ArcadeValues {
    /// 1 when the front end's scene was let go for a race (`menu::reload_front_end` sets it up
    /// again).
    pub scene_let_go: u32,
    /// The arcade level being played.
    pub level: u32,
    /// The arcade stage being played.
    pub stage: u32,
    /// 1 after a lost level (the retry screens follow).
    pub retry: u32,
    /// The places the arcade game ranks.
    pub places: u32,
    /// One byte a level: 1 when the level is open (the options file keeps the first six).
    pub levels_won: [u8; 8],
}

impl Arcade {
    pub const fn new() -> Self {
        Arcade(GameThread::new(RefCell::new(ArcadeValues {
            scene_let_go: 0x0,
            level: 0x0,
            stage: 0x1,
            retry: 0x0,
            places: 0x64,
            levels_won: [0; 8],
        })))
    }

    /// A copy of the values: no borrow outlives the statement that reads them.
    pub fn read(&self) -> ArcadeValues {
        *self.0.get().borrow()
    }

    /// The values, to write.
    pub fn write(&self) -> RefMut<'_, ArcadeValues> {
        self.0.get().borrow_mut()
    }

    /// The level and the stage being played.
    pub fn position(&self) -> (u32, u32) {
        let values = self.read();
        (values.level, values.stage)
    }

    /// Starts `level` at its first stage.
    pub(super) fn start_level(&self, level: u32) {
        let mut values = self.write();
        values.level = level;
        values.stage = 0;
    }

    /// The opened byte of level `index` (0 based; the level after `index` is opened by winning
    /// it): 1 when open.
    pub fn opened(&self, index: usize) -> u8 {
        self.read().levels_won[index]
    }

    /// Marks level `index` (0 based) open.
    pub fn open(&self, index: usize) {
        self.write().levels_won[index] = 1;
    }
}
