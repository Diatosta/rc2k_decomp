//! `menu::mark_stage_choice`: `champ_view.stage_mode` 1 (the stage screens' map then takes
//! the choice's stage; 2 a championship's, 3 a record's).
//!
//! One caller, `menu::select_stage_loop` at its set-up. Rust callers run its `entry` (`routes.rs`);

use crate::menu::champ_store::ChampView;

pub fn mark_stage_choice(champ: &ChampView) {
    champ
        .stage_mode
        .set(rc2k_core::menu::event::MapStage::Choice as u32);
}
