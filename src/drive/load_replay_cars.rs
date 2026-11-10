//! `drive::load_replay_cars` (0x46010D): `drive::save_replay_cars` the other way: the replay's
//! starting state at `unk_7172EC` copied back into the four car blocks, the race slots and
//! `unk_71BF84`.
//!
//! Callers: `drive::play_replay_frame`, `race::show_replay_still` (twice).

use crate::drive::save_replay_cars::copy_piece;
use crate::race::owner::Race;
use rc2k_core::drive::replay_frames::start_layout;

// Converted: `rc2k_core::drive::replay_frames::start_layout`.
pub fn load_replay_cars(race: &Race) {
    for (piece, offset, n) in start_layout() {
        copy_piece(race, piece, offset, n, false);
    }
}
