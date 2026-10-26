//! `net::choose_connection` (0x4174C5): the list screen of the DirectPlay connections
//! (`net::connections_frame` through `screens::run_frame`): the clocks from `dword_518998`,
//! the top and the highlight 0; each frame a key (`menu::list_take_key`): up (0x66) and down
//! (0x6B) move the highlight (within 0 .. the count - 1, signed) and the top with it
//! (sixteen rows), Enter (0x1C) chooses, Escape (1) or the game closing gives -1; 'R' takes a
//! screenshot (with screenshots on), any other key may compile the lexicon again.
//!
//! In:  nothing
//! Out: eax = the connection chosen (`g_listChoice`), carry set if negative (else the flags
//!      of `test eax, eax`); the rest kept (`pusha` .. `popa`)
//! Callers: `net::start_link`, `net::settings_link`. Its calls through `part` (the frames
//! through the game). The whole screen runs inside it: ungated. Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins (the keys scripted),
//! against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    CHOICE, CHOICE_BEFORE, CLOCK, CONNECTIONS, FRAME_CLOCK, KEY_CLOCK, LIST_TAKE_KEY,
    MAYBE_LEXICON, MOVE_CLOCK, Part, QUITTING, RUN_FRAME, SCREENSHOT, SCREENSHOTS_ON, g, game,
    lobby_entry, p, test_eax,
};
use rc2k_core::net::screens::{self, ListKey};

const FRAME: u32 = 0x4175C7; // net::connections_frame

/// A list's up (0x66) or down (0x6B) on a list of `count` items: the highlight and the top
/// moved, as the original's eax and ebx leave them; false for another key.
pub fn list_move(r: &mut Regs, key: u8, count: u32) -> bool {
    // Converted: `rc2k_core::net::screens` (`ListKey`, `up`, `down`).
    match ListKey::of(key) {
        ListKey::Up => {
            let (choice, top) = screens::up(g(CHOICE), g(CHOICE_BEFORE));
            p(CHOICE, choice);
            p(CHOICE_BEFORE, top);
            r.eax = top;
            true
        }
        ListKey::Down => {
            let (choice, top) = screens::down(g(CHOICE), g(CHOICE_BEFORE), count);
            p(CHOICE, choice);
            p(CHOICE_BEFORE, top);
            r.eax = top;
            r.ebx = choice.wrapping_sub(screens::LIST_ROWS - 1);
            true
        }
        _ => false,
    }
}

/// The end of the list screens: eax the choice, carry if negative.
pub fn chosen(entry: &Regs, r: &Regs) -> Regs {
    let mut left = Regs {
        eflags: r.eflags,
        ..*entry
    };
    left.eax = g(CHOICE);
    test_eax(&mut left);
    if (left.eax as i32) < 0 {
        left.set_carry(true);
    }
    left
}

/// The Rust version: the registers it leaves.
pub fn choose_connection(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = g(CLOCK);
    p(MOVE_CLOCK, r.eax);
    p(KEY_CLOCK, r.eax);
    p(CHOICE_BEFORE, 0);
    p(CHOICE, 0);
    loop {
        r.eax = game(FRAME);
        part(RUN_FRAME, &mut r)?;
        r.eax = g(CLOCK);
        p(FRAME_CLOCK, r.eax);
        part(LIST_TAKE_KEY, &mut r)?;
        let key = r.eax as u8;
        if list_move(&mut r, key, g(CONNECTIONS)) {
            r.ebx = g(FRAME_CLOCK);
            p(MOVE_CLOCK, r.ebx);
        } else if ListKey::of(key) == ListKey::Enter {
            r.eax = g(CHOICE);
            return Ok(chosen(entry, &r));
        } else if ListKey::of(key) == ListKey::Escape {
            break;
        } else {
            if g(SCREENSHOTS_ON) != 0 && key == 0x52 {
                part(SCREENSHOT, &mut r)?;
            }
            part(MAYBE_LEXICON, &mut r)?;
        }
        if g(QUITTING) == 1 {
            break;
        }
    }
    p(CHOICE, u32::MAX);
    Ok(chosen(entry, &r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, choose_connection)
}
