//! The two replay starts the review menus' items lead to (`race::replay_mode_1`,
//! `race::replay_mode_3`): with a replay there (`dword_5BEC84` not 0), the replay viewer
//! `race::replay_viewer` (0x41CF0C) with eax the mode.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};

pub const REPLAY_RVA: usize = rva(0x5BEC84); // dword_5BEC84: a replay there
// sub_41CF0C

/// A call by place: 0 the viewer (eax the mode).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// One start: the registers it leaves.
pub unsafe fn replay_mode(entry: &Regs, mode: u32, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::views_replay`.
    let mut r = *entry;
    if rc2k_core::race::viewer_flow::views_replay(unsafe { game_read::<u32>(REPLAY_RVA) }) {
        r.eax = mode;
        part(0, &mut r)?;
    }
    Ok(r)
}

/// In the game: the viewer's Rust version (`race::replay_viewer`).
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::replay_viewer::{game_part, replay_viewer};
    *r = replay_viewer(r, &mut game_part)?;
    Ok(())
}
