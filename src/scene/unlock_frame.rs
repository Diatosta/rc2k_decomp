//! `scene::unlock_frame` (0x44E851): the back surface unlocked (`gfx::unlock_surface`, the
//! platform's) and the kept pitch and pixels (`dword_5C5FB4`, `dword_5C5FB0`) 0.
//!
//! In:  nothing
//! Out: CF as the unlock leaves it; eax 0; the rest as the unlock leaves them
//!
//! Callers: `race::loading_bar_frame` and three of the game's. The registers above are the
//! original's; a call by address goes through `routes.rs`, which maps them.

use crate::framework::{game_write, rva};

/// The back surface unlocked and the kept pitch and pixels 0. Returns `g_surfaceLocked`
/// before (the original leaves the flags of comparing it with 1, and eax 0).
pub fn unlock_frame() -> u32 {
    let locked = crate::platform::unlock_surface();
    unsafe {
        game_write(rva(0x5C5FB4), 0u32);
        game_write(rva(0x5C5FB0), 0u32);
    }
    locked
}
