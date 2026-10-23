//! `race::race_keys` (0x43A649): the race's view and sound keys, each frame.
//!
//! In:  nothing
//! Out: eax, esi (the view block), ebx (on the next-view key) as it leaves them; the rest
//!      kept (both callers go on with a `call` or a `mov`: no flags read)
//!
//! `drive::choose_shown_cars`; esi the first view's block (`off_63DBB0`), the second's
//! (`off_63DBB4`) with `byte_4F6B79` set. Then the first key down, in this order:
//! - `byte_4F6D17`: `drive::next_camera_view`, then `set_car_counter` with
//!   `dword_6138D4[view]` when the view (+0x688) is below 7;
//! - `byte_4F6D18` (not with one player): `dword_6238A4 ^= 3`, `set_both_counters` 0x127
//!   (now 1) or 0x128;
//! - `byte_4F6D19` / `byte_4F6D1A`: the volume down / up by 4 (`& !3`, held to 0 ..= 0x40):
//!   `g_masterVolume` then `sound::send_command_8` (counters 0x113 / 0x114); with
//!   `byte_4F6B79` set `dword_518100` then `sound::set_music_volume` (0x115 / 0x116; the
//!   "up" step starts from `dword_75568C`, a slip of the original's: `notes/bugs.md`);
//! - seven keys that flip a bit of the block's +0x220, each with `set_car_counter` (see
//!   `BIT_KEYS`);
//! - `byte_4F6D1B`: bit 0x40 of the first view's block, `set_both_counters` 0x125 / 0x126.
//!
//! Refuses (before anything) what `choose_shown_cars` refuses. Starts with a call:
//! private copy (every call relocated), manual patch. Shadow mode: the original runs (its
//! sounds sent), the memory put back, the Rust version runs without sending, the memory
//! and registers compared, the original's put back. Self-test: a copy whose two sound calls
//! go to stand-ins that note eax .. ecx, against the Rust version on the same, every key
//! with both views, players, volumes at the ends; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn all(after: &Regs) -> Left {
    use crate::framework::{EAX, EBP, EBX, ECX, EDI, EDX, ESI};
    Left::nothing()
        .with(EAX, after.eax)
        .with(EBX, after.ebx)
        .with(ECX, after.ecx)
        .with(EDX, after.edx)
        .with(ESI, after.esi)
        .with(EDI, after.edi)
        .with(EBP, after.ebp)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let after = unsafe { race_keys(r, &mut key_sound_part)? };
    Ok(all(&after))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// dword_63DB94
// dword_52A910
// dword_75568C
