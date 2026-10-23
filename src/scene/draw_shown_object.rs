//! `scene::draw_shown_object` (0x4503A5): draws the object `g_objectNumber` unless a
//! race's tests hide it.
//!
//! In:  nothing
//! Out: the registers as its parts leave them (after a drawing, `scene::draw_object`'s)
//!
//! `dword_62DA44` = 1 for the duration, 0 at the end.
//! In a race (`g_inRace` not 0) two tests, each of which can hide the object:
//! - `scene::object_hidden_by_kind` (0x4522EA over 0x452303, inlined): with
//!   `dword_71C3E0` not 0 or `dword_62389C` at 2, an object of kind 0x30 or 0x4E;
//! - `scene::object_hidden` (0x4503DD, inlined; leaves ebx = the object's number):
//!   objects 0x38 and 0x3A .. 0x3F are hidden with `dword_621480` set, with
//!   `dword_609330` at 0, object 0x3E with `dword_71B400` not 0 and object 0x3D with it
//!   not 1; with `dword_621480` set and `dword_635058` not 0 (eax), every object unless
//!   `dword_635058` - 1 is `dword_71B400`; and an object without faces
//!   (`g_objectFaceCounts`).
//!
//! Outside a race `scene::clear_kind_bytes` (0x46219C, inlined): the 0x100 dwords at
//! `dword_5CD688` = 0, then the seventh, eighth and tenth = 999 (eax = 0, ecx = 0,
//! edi = the table's end).
//! Then `scene::draw_object`.
//! One caller, `sub_451E3E`.
//! Refuses, in a race, an object's number of 0x100 or more (the original reads past
//! its tables).
//! Shadow mode runs the original alone and compares whether it drew with what the Rust
//! tests say (through `draw_object`'s hook, while that is on).
//! Self-test: a redirected copy whose drawing notes itself, with the original's own
//! tests, on settings drawn by lot.
//!
//! Converted 2026-10-21: the race's tests and the kind bytes through
//! `rc2k_core::scene::shown` (the variables read where the tests read them).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

// ---- game data ----

const DRAWING_RVA: usize = rva(0x62DA44); // dword_62DA44
const IN_RACE_RVA: usize = rva(0x5C5CEC); // g_inRace
const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const KIND_SWITCH_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const KIND_MODE_RVA: usize = rva(0x62389C); // dword_62389C
const KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds
const OTHER_WAY_RVA: usize = rva(0x621480); // dword_621480
const SHOWN_RVA: usize = rva(0x609330); // dword_609330
const WHICH_RVA: usize = rva(0x71B400); // dword_71B400
const ONLY_RVA: usize = rva(0x635058); // dword_635058
const FACE_COUNTS_RVA: usize = rva(0x632F70); // dword_632F70
const KIND_BYTES_RVA: usize = rva(0x5CD688); // dword_5CD688
use rc2k_core::scene::shown::OBJECTS;

// ---- Rust version ----

/// The game's variables for the race's tests.
struct Game;

impl rc2k_core::scene::shown::RaceObject for Game {
    fn object(&self) -> u32 {
        unsafe { game_read(OBJECT_RVA) }
    }
    fn kind_switch(&self) -> u32 {
        unsafe { game_read(KIND_SWITCH_RVA) }
    }
    fn kind_mode(&self) -> u32 {
        unsafe { game_read(KIND_MODE_RVA) }
    }
    fn kind(&self, object: u32) -> u32 {
        unsafe { game_read(KINDS_RVA + object as usize * 4) }
    }
    fn other_way(&self) -> u32 {
        unsafe { game_read(OTHER_WAY_RVA) }
    }
    fn shown(&self) -> u32 {
        unsafe { game_read(SHOWN_RVA) }
    }
    fn which(&self) -> u32 {
        unsafe { game_read(WHICH_RVA) }
    }
    fn only(&self) -> u32 {
        unsafe { game_read(ONLY_RVA) }
    }
    fn faces(&self, object: u32) -> u32 {
        unsafe { game_read(FACE_COUNTS_RVA + object as usize * 4) }
    }
}

/// The race's two tests: whether the object is drawn (its number checked by the caller).
/// Writes nothing but the registers the tests leave.
// Converted: `rc2k_core::scene::shown::race_shown`
fn shown(r: &mut Regs) -> bool {
    let s = rc2k_core::scene::shown::race_shown(&Game);
    r.ebx = s.ebx.unwrap_or(r.ebx);
    r.eax = s.eax.unwrap_or(r.eax);
    s.drawn
}

/// `draw`: what stands for `scene::draw_object`. Says whether the object was drawn.
pub unsafe fn draw_shown_object(
    r: &mut Regs,
    draw: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<bool, Refused> {
    unsafe {
        let race = game_read::<u32>(IN_RACE_RVA) != 0;
        if race && game_read::<u32>(OBJECT_RVA) >= OBJECTS {
            return Err(BadInput("an object's number past the tables").into());
        }
        game_write(DRAWING_RVA, 1u32);
        let show = if race {
            shown(r)
        } else {
            game_write(KIND_BYTES_RVA, rc2k_core::scene::shown::kind_bytes());
            r.eax = 0;
            r.ecx = 0;
            r.edi = (game_addr(KIND_BYTES_RVA) + 0x400) as u32;
            true
        };
        if show {
            draw(r)?;
        }
        game_write(DRAWING_RVA, 0u32);
        Ok(show)
    }
}
