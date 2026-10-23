//! `race::draw_viewer_item` (0x41D94A): one item of the replay viewer's list drawn, unless
//! hidden (bit 6): its picture (`[esi+0xC]`, a texture rectangle; for the three switch
//! items, `off_5C2C70` by the block's camera byte 2 for item kind 0xE or 3 otherwise,
//! `off_5C2C78` by byte 5, `off_5C2C88` by byte 4, the table's entry) as a quad
//! (`drive::tyre_quad`) on the item's place (`[esi+4]`: x, y, w, h) at the picture's
//! width (its absolute value), centred (`dword_5C29FC`); grey (0x666666) when marked
//! unusable (bit 7), else white, or for the highlighted item (ecx `dword_51D407`) red
//! while a camera is edited, else yellow and red by turns (bit 3 of `dword_51D41F -
//! dword_51D423`).
//!
//! In:  esi the item (16 bytes), ecx its number
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::viewer_frame` (each item). Its call through `part` (1 `tyre_quad`, eax
//! the picture, ebx the place, ecx the colour; in the game its Rust version). Shadow mode
//! runs the original alone (it draws). Self-test: a copy whose call goes to a stand-in,
//! against the Rust version: items hidden, unusable, highlighted (edited or not, both
//! turns), plain, and the three switch items with their bytes.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::{a, g, p};
use crate::race::viewer::{
    BLOCK, CAMERA_EDIT, CAMERAS, CHOICE, FRAME_CLOCK, MOVE_CLOCK, gb, regs_entry,
};

const SWITCH_A: usize = 0x5C2C70; // off_5C2C70: by camera byte 2 or 3
const SWITCH_B: usize = 0x5C2C78; // off_5C2C78: by byte 5
const SWITCH_C: usize = 0x5C2C88; // off_5C2C88: by byte 4
const PLACE: usize = 0x5C29FC; // dword_5C29FC: the quad's place

fn camera_byte(byte: usize) -> u32 {
    gb((CAMERAS + byte).wrapping_add(8 * g(BLOCK) as usize)) as u32
}

/// The Rust version: the registers it leaves (the entry's).
pub fn draw_viewer_item(entry: &Regs) -> Result<Regs, Refused> {
    let item = entry.esi;
    let byte = |n: u32| unsafe { ((item + n) as *const u8).read() };
    let word = |at: u32| unsafe { (at as *const u32).read_unaligned() };
    if byte(0) & 0x40 != 0 {
        return Ok(*entry);
    }
    let mut picture = word(item + 0xC);
    // Converted: `rc2k_core::race::viewer::{switch_byte, centred, item_colour}`.
    use rc2k_core::race::viewer::{centred, item_colour, switch_byte};
    if picture == a(SWITCH_A) {
        let n = camera_byte(switch_byte(byte(1)) as usize);
        picture = word(picture.wrapping_add(4 * n));
    }
    if picture == a(SWITCH_B) {
        picture = word(picture.wrapping_add(4 * camera_byte(5)));
    }
    if picture == a(SWITCH_C) {
        picture = word(picture.wrapping_add(4 * camera_byte(4)));
    }
    let place = word(item + 4);
    let width = (word(picture + 8) as i32).unsigned_abs();
    p(PLACE + 8, width);
    p(PLACE, centred(word(place), word(place + 8), width));
    p(PLACE + 4, word(place + 4));
    p(PLACE + 0xC, word(place + 0xC));
    let colour = item_colour(
        byte(0),
        entry.ecx == g(CHOICE),
        || g(CAMERA_EDIT) != 0,
        || g(FRAME_CLOCK).wrapping_sub(g(MOVE_CLOCK)),
    );
    let mut r = Regs {
        eax: picture,
        ebx: a(PLACE),
        ecx: colour,
        edx: picture,
        edi: place,
        ..*entry
    };
    crate::race::draw_viewer_item::game_part(1, &mut r)?;
    Ok(*entry)
}

/// In the game: `tyre_quad`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    *r =
        unsafe { crate::drive::tyre_quad::tyre_quad(r, &mut crate::drive::tyre_parts::game_part) }?;
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| draw_viewer_item(r))
}
