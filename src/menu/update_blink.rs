//! `menu::update_blink` (0x47E826): once a frame, works out from the clock what blinks and
//! whether the game has been left alone for a long time.
//!
//! In:  nothing
//! Out: every register preserved
//! After `update_frame_clock`:
//!   - `dword_7559A4` (the highlight's blink, as the service area's markers use it) is 1 while
//!     bit 8 of `clock - dword_7556C4` is clear, so it is on for 256 ms and off for 256 ms,
//!     counted from the last key; it stays 0 while `dword_7556AC` is 1;
//!   - `dword_7559A8` (the text entry's cursor) is 1 while bit 9 of `clock - dword_7556C8` is
//!     clear: 512 ms on, 512 ms off;
//!   - `dword_755551` is 1 once `clock - dword_7556C4` has reached 300000: five minutes
//!     without a key.
//!
//! 179 callers. Shadow mode lets the original run and checks what it stored against the Rust
//! arithmetic on the clock value it used.
//! Manual patch with a private copy (0x95 bytes, one call relocated): a call sits in the
//! first bytes.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, game_write, rva};
use crate::menu::update_frame_clock::{CLOCK_RVA, update_frame_clock};

// ---- game data ----

pub const LAST_KEY_TIME_RVA: usize = rva(0x7556C4); // dword_7556C4: the clock at the last key
const CURSOR_TIME_RVA: usize = rva(0x7556C8); // dword_7556C8: what the cursor's blink counts from
pub const BLINK_OFF_RVA: usize = rva(0x7556AC); // dword_7556AC: 1 = the highlight does not blink
const BLINK_RVA: usize = rva(0x7559A4); // dword_7559A4
const CURSOR_ON_RVA: usize = rva(0x7559A8); // dword_7559A8
const IDLE_RVA: usize = rva(0x755551); // dword_755551

// ---- Rust version ----

// Converted: `rc2k_core::menu::blink::blink_states`.
pub use rc2k_core::menu::blink::blink_states;

/// What the three variables should be for the clock the game holds now.
unsafe fn states_now() -> (u32, u32, u32) {
    unsafe {
        blink_states(
            game_read(CLOCK_RVA),
            game_read(LAST_KEY_TIME_RVA),
            game_read(CURSOR_TIME_RVA),
            game_read(BLINK_OFF_RVA),
        )
    }
}

pub fn update_blink() {
    update_frame_clock();
    unsafe {
        let (blink, cursor_on, idle) = states_now();
        game_write(BLINK_RVA, blink);
        game_write(CURSOR_ON_RVA, cursor_on);
        game_write(IDLE_RVA, idle);
    }
}
