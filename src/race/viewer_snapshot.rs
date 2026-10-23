//! `race::viewer_snapshot` (0x41EFAB): the viewer's frame saved as a picture: the three
//! digits of `savedata\snapshot\000.pcx` one more (999 round to 000; the number found by
//! `race::find_snapshot_number`), the name as `dword_4F49C8` while the surface is locked
//! (`gfx::lock_surface`; failing, nothing), written (`gfx::write_screen_pcx`) and
//! unlocked (`gfx::unlock_surface`); `dword_4F49C8` put back.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read (the caller `test`s eax)
//!
//! Caller: `race::viewer_frame` (when `dword_5C2540` asks). Its calls through `part`: 1 the
//! lock (carry), 2 the writer, 3 the unlock; in the game as `gfx::take_screenshot`'s. Shadow
//! mode runs the original alone (it writes a file). Self-test: a copy whose calls go to
//! stand-ins, against the Rust version: digits to carry, the lock failing.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::gfx::write_screen_pcx::PICTURE_NAME;
use crate::physics::flow::a;
use crate::race::find_snapshot_number::{SNAPSHOT_DIGITS, SNAPSHOT_PATH};
use crate::race::viewer::{Part, get, put, regs_entry};

/// The Rust version: the registers it leaves (the entry's).
pub fn viewer_snapshot(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let kept = get(PICTURE_NAME);
    let digits = a(SNAPSHOT_DIGITS) as *mut u8;
    let mut r = Regs {
        eax: a(SNAPSHOT_PATH),
        ..*entry
    };
    // Converted: `rc2k_core::race::viewer::next_snapshot`.
    unsafe {
        let (next, last, ecx) =
            rc2k_core::race::viewer::next_snapshot(digits.cast::<[u8; 3]>().read());
        digits.cast::<[u8; 3]>().write(next);
        r.ebx = digits.add(last) as u32;
        r.ecx = ecx;
    }
    put(PICTURE_NAME, r.eax);
    part(1, &mut r)?;
    if !r.carry() {
        part(2, &mut r)?;
        part(3, &mut r)?;
    }
    put(PICTURE_NAME, kept);
    Ok(*entry)
}

/// In the game: as `gfx::take_screenshot`'s.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::gfx::take_screenshot::game_part(place, r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| viewer_snapshot(r, &mut game_part))
}
