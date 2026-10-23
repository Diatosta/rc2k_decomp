//! The two "edit a camera" actions of the replay viewer, and the camera changes they lead
//! to (IDA 2026-10-19):
//! - `race::edit_camera_1` (0x41E599), `race::edit_camera_2` (0x41E5BA): the replay kept
//!   for an undo (`race::keep_undo`); unless the block shown (`dword_5C2530`) is the last
//!   (`dword_5BECD0`), `dword_96E480` 1 or 2: the keys then change that camera.
//! - `race::next_camera` (0x41E5EB), `race::previous_camera` (0x41E654): unless the block
//!   shown is the last, its camera byte `dword_96E480 - 1` (`unk_963BB3[edit + block * 8]`)
//!   one on (past 0x2E back to 0) or back (below 0 to 0x2E), skipping the cameras whose
//!   name (`word_5C291C`) is 0x1C1 or 0x1C2 unless the replay is one player's
//!   (`dword_719B34` 1) with a first car outside 0x17 .. 0x20; then the change spread on
//!   (`race::spread_camera`, dh the old value).

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::a;
use crate::race::viewer::{
    BLOCK, BLOCKS, CAMERA_EDIT, CAMERA_NAMES, FIRST_CAR, ONE_PLAYER, get, put,
};

/// `edit_camera_1` / `_2`: eax the block shown.
pub fn edit_camera(entry: &Regs, camera: u32) -> Regs {
    // Converted: `rc2k_core::race::viewer_flow::edits_camera`.
    let mut r = *entry;
    crate::race::keep_undo::keep_undo();
    r.eax = get(BLOCK);
    if rc2k_core::race::viewer_flow::edits_camera(r.eax, get(BLOCKS)) {
        put(CAMERA_EDIT, camera);
    }
    r
}

fn name_of(camera: u8) -> u16 {
    unsafe { (a(CAMERA_NAMES + 2 * camera as usize) as *const u16).read_unaligned() }
}

/// `next_camera` (`step` 1) and `previous_camera` (-1): the registers they leave.
pub unsafe fn step_camera(entry: &Regs, step: i8) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = get(BLOCK);
    if r.eax == get(BLOCKS) {
        return Ok(r);
    }
    r.ebx = a(0x963BB3)
        .wrapping_add(get(CAMERA_EDIT))
        .wrapping_add(r.eax.wrapping_mul(8));
    let at = r.ebx as *mut u8;
    let old = unsafe { at.read() };
    // Converted: `rc2k_core::race::viewer::{every_camera, step_camera}`.
    use rc2k_core::race::viewer::{camera_skipped, every_camera, step_camera};
    let every = every_camera(get(ONE_PLAYER) == 1, get(FIRST_CAR));
    let dl = step_camera(old, step > 0, every, |n| camera_skipped(name_of(n)))?;
    if !every {
        r.eax = dl as u32;
    }
    unsafe { at.write(dl) };
    r.edx = r.edx & !0xFFFF | (old as u32) << 8 | dl as u32;
    let spread = unsafe { crate::race::spread_camera::spread_camera(&r) };
    if spread.is_err() {
        unsafe { at.write(old) }; // refused before it wrote: nothing left written
    }
    spread
}
