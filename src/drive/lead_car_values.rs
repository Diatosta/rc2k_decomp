//! The eight fields of car block 0 that `drive::share_all_cars` keeps around its work:
//! (block field, keeping place) pairs, in the original's order.
use crate::race::owner_frame::Frame;
use crate::session::Dword;

/// (car block 0's field, its keeping place): the camera's view, place and
/// turns, and the 2-or-3 mode flag.
pub fn pairs(frame: &Frame) -> [(&Dword, &Dword); 8] {
    let lead = crate::race::blocks_store::own_car();
    let kept = &frame.frame_work.lead_kept;
    [
        (&lead.camera_view, &kept[0]),
        (&lead.camera_position[0], &kept[1]),
        (&lead.camera_position[1], &kept[2]),
        (&lead.camera_position[2], &kept[3]),
        (&lead.camera_pitch, &kept[4]),
        (&lead.camera_heading, &kept[5]),
        (&lead.camera_roll_turn, &kept[6]),
        (&lead.camera_mode_is_2_or_3, &kept[7]),
    ]
}
