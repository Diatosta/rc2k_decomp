//! `audio::stop_streams` (0x40C5E8): both stream commands with eax 0.
//!
//! In:  ebx, ecx, edx = passed on (callers: the thread's end 0x409DD6, the mode change
//!      0x40A43C)
//! Out: every register preserved
//!
//! `audio::stream_command` and then `audio::music_command`, each with eax 0 and the
//! caller's ebx, ecx and edx: with action 0 (or any but 1 and 3) both stop. Shadow mode
//! runs the original alone. Self-test: a redirected copy whose two calls go to stand-ins
//! noting their registers, 4 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::stream::SOUND_STREAM_RVA;
use crate::audio::stream_commands::{MusicVars, game_part, music_command, stream_command};
use crate::errors::Refused;
use crate::framework::{Regs, game_addr};

/// Its two calls: 0 `stream_command`, 1 `music_command`, with the registers they get.
pub fn stop_streams(
    r: &Regs,
    command: &mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    for which in 0..2 {
        let mut c = Regs { eax: 0, ..*r };
        command(which, &mut c)?;
    }
    Ok(())
}

pub fn game_command(which: u32, r: &mut Regs) -> Result<(), Refused> {
    if which == 0 {
        let player = game_addr(SOUND_STREAM_RVA) as u32;
        stream_command(r.eax, r.ecx as i32, r.edx, player, &mut game_part)
    } else {
        music_command(
            r.eax,
            r.ebx,
            r.ecx as i32,
            r.edx,
            &MusicVars::game(),
            &mut game_part,
        )
    }
}
