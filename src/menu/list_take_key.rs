//! `menu::list_take_key` (0x41469A): a list menu's key: the pending key taken
//! (`input::take_key`, eax 0 first); none: a millisecond's sleep (`system::sleep`). A key:
//! the repeat count `dword_51D40F` reset unless it is the last key again (`byte_51D413`)
//! and `dword_51D41F - (dword_51D423 + 6)` is negative (`js`), then one more; the key
//! into `byte_51D413`.
//!
//! In:  nothing
//! Out: eax the key (0 for none); with the last key again ebx `dword_51D41F` and ecx
//!      `dword_51D423 + 6`; the flags of `test al, al` (no caller reads them)
//!
//! Callers: `race::replay_viewer`, `sub_4144CF`, `sub_4168CF`, `sub_4174C5`,
//! `sub_417726`, `sub_417A0D`. Its two calls through `part`: 1 `input::take_key`, 2
//! `system::sleep` (eax ms); in the game their Rust versions. Shadow mode: the original
//! first on the real key, the Rust version on the same key replayed. Self-test: a copy
//! whose two calls go to stand-ins (the key given), against the Rust version: no key, a
//! new key, the last key again with the clocks either side of the sign.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::viewer::{FRAME_CLOCK, LAST_KEY, MOVE_CLOCK, REPEATS, gb, get, left_of, pb, put};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version: the registers it leaves (the flags of `test al, al`).
pub fn list_take_key(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = 0;
    crate::menu::list_take_key::game_part(1, &mut r)?;
    let key = r.eax as u8;
    if key == 0 {
        let mut s = Regs { eax: 1, ..r };
        crate::menu::list_take_key::game_part(2, &mut s)?;
    } else {
        // Converted: `rc2k_core::menu::list_menu::repeats`.
        let again = key == gb(LAST_KEY);
        let (mut frame, mut moved) = (0, 0);
        if again {
            (frame, moved) = (get(FRAME_CLOCK), get(MOVE_CLOCK));
            r.ebx = frame;
            r.ecx = moved.wrapping_add(6);
        }
        put(
            REPEATS,
            rc2k_core::menu::list_menu::repeats(get(REPEATS), again, frame, moved),
        );
        pb(LAST_KEY, key);
    }
    r.set_carry(false);
    r.set_zero(key == 0);
    Ok(r)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 1 {
        let key = crate::input::take_key::take_key();
        r.eax = r.eax & !0xFF | key as u32;
    } else {
        crate::system::sleep::sleep(r.eax);
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&list_take_key(r)?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
