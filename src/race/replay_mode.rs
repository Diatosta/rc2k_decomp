//! The two replay starts the review menus' items lead to (`race::replay_mode_1`,
//! `race::replay_mode_3`): with a replay there (`dword_5BEC84` not 0), the replay viewer
//! `race::replay_viewer` (0x41CF0C) with eax the mode.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};

pub const REPLAY_RVA: usize = rva(0x5BEC84); // dword_5BEC84: a replay there
pub const VIEWER: usize = 0x41CF0C; // sub_41CF0C

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
    *r = replay_viewer(r, &mut game_part, true)?;
    Ok(())
}

/// The self-test of either: a copy whose viewer call goes to a stand-in, against the Rust
/// version, with and without a replay.
pub fn test_mode(hook: &crate::framework::RawHook, address: usize, mode: u32) -> (usize, usize) {
    use crate::framework::{SelfTest, game_write};
    use crate::service::harness::{at, capture, restore};
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    // cmp dword_5BEC84, 0
    let copy = match crate::race::mud::copy(
        address,
        0x14,
        &[0x83, 0x3D, 0x84, 0xEC, 0x5B, 0x00, 0x00],
        &[0xE],
        &[(VIEWER, 0)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [at(REPLAY_RVA)];
    let saved = capture(&regions);
    for there in [0u32, 1, 7] {
        crate::race::mud::run_both(
            &mut t,
            &format!("replay there {there}"),
            &copy,
            &regions,
            &|| unsafe { game_write(REPLAY_RVA, there) },
            &|r| unsafe { replay_mode(r, mode, &mut crate::race::mud::test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
