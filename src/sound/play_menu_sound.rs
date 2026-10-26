//! `sound::play_menu_sound` (0x40F059): plays one of the menus' sounds.
//!
//! In:  eax = which, as bits; esi goes into the command as it is
//! Out: every register preserved
//! Nothing for eax = 0, and nothing before the time kept in `dword_511B0C` (125 ms after
//! the last sound; compared by the sign of the difference). The time is moved on before
//! the bits are looked at. The first bit set of these, in this order, picks the sound
//! and the volume it is played at:
//! 0x80 sound 0xE4 at `dword_755694`; 0x10, 0x20, 0x40 sounds 0x163, 0x197, 0x1CB at
//! `dword_7554E8`; 0x100, 8, 2, 4, 1 sounds 0x1CC, 0x1CA, 0x1C8, 0x1C9, 0x1C7 at
//! `dword_755690`. Then `sound::play_sound` with edx = 0xFFFFFF91 and ecx, edi, ebp = 0.
//! 25 callers (1 is the highlight's click, 2 Enter's sound: `menu::handle_keys`).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_read, game_write, rva};
use crate::sound::play_sound::play_sound_in_game;
use crate::system::tick_count::tick_count;

// ---- game data ----

const NEXT_TIME_RVA: usize = rva(0x511B0C); // dword_511B0C
const VOLUME_A_RVA: usize = rva(0x755694); // dword_755694
const VOLUME_B_RVA: usize = rva(0x7554E8); // dword_7554E8
const VOLUME_C_RVA: usize = rva(0x755690); // dword_755690

// ---- Rust version ----

/// `now` is asked for the clock; `play` gets the registers `sound::play_sound` is
/// called with.
pub unsafe fn play_menu_sound(
    entry: &Regs,
    now: &mut dyn FnMut() -> u32,
    play: &mut dyn FnMut(&Regs),
) {
    unsafe {
        // Converted: `rc2k_core::sound::senders::menu_sound`.
        use rc2k_core::sound::senders::{MenuVolume, menu_sound};
        if entry.eax == 0 {
            return;
        }
        let Some((next, pick)) = menu_sound(entry.eax, now(), game_read(NEXT_TIME_RVA)) else {
            return;
        };
        game_write(NEXT_TIME_RVA, next);
        let Some((sound, volume)) = pick else {
            return;
        };
        let volume = match volume {
            MenuVolume::A => VOLUME_A_RVA,
            MenuVolume::B => VOLUME_B_RVA,
            MenuVolume::C => VOLUME_C_RVA,
        };
        play(&Regs {
            eax: sound,
            ebx: game_read(volume),
            ecx: 0,
            edx: 0xFFFF_FF91,
            edi: 0,
            ebp: 0,
            ..*entry
        });
    }
}

/// With the game's clock and queue: what other Rust code calls.
pub fn play_menu_sound_in_game(entry: &Regs) {
    unsafe { play_menu_sound(entry, &mut tick_count, &mut |with| play_sound_in_game(with)) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    play_menu_sound_in_game(r);
    Ok(())
}
