//! The eight fields of car block 0 that `drive::share_all_cars` keeps around its work:
//! (block field, keeping place) pairs, in the original's order.

use crate::race::blocks_store::CAR_BLOCKS;
use crate::session::Dword;

/// (car block 0's field, its keeping place's IDA address): the camera's view, place and
/// turns, and the 2-or-3 mode flag.
pub fn pairs() -> [(&'static Dword, usize); 8] {
    let lead = &CAR_BLOCKS.cars[0];
    [
        (&lead.camera_view, 0x64165C),
        (&lead.camera_position[0], 0x641660),
        (&lead.camera_position[1], 0x641664),
        (&lead.camera_position[2], 0x641668),
        (&lead.camera_pitch, 0x64166C),
        (&lead.camera_heading, 0x641670),
        (&lead.camera_roll_turn, 0x641674),
        (&lead.camera_mode_is_2_or_3, 0x641678),
    ]
}
