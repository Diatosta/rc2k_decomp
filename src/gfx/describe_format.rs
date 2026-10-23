//! `gfx::describe_format` (0x405BB6): works a pixel format's layout out from its masks.
//!
//! In:  ebx = the format (`PixelFormat`: +0 layout, +4 four masks, +0x14 number)
//! Out: every register preserved; +0 and +0x14 written
//! If each of the four masks is empty or one run of bits (`sub_405C79`): the layout is
//! four bytes, lowest first the masks' top bits' numbers less 7, 15, 23 and 31 (an empty
//! mask counts as 0xFF; `sub_405C65`), with the sign bit set: the rotation counts the
//! colour conversions use. If the four masks together are one run as well, the format's
//! number is looked up by that layout in the table of six at `dword_4F3F78` (number,
//! layout: 1 .. 4 the 16-bit formats, 5 and 6 the 32-bit ones); else, and when it is
//! not there, the number is 0.
//! If a mask has a gap: the layout is the number of the top bit of all masks together,
//! and the number 0.
//! Three callers: `gfx::set_component_format`, the screen mode's set-up at 0x404EF2 and
//! the textures' at 0x41A1E7.
//! Refuses a null format.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

const NUMBERS_RVA: usize = rva(0x4F3F78); // dword_4F3F78: six of (number, layout)
const NUMBERS: usize = 6;
// ---- Rust version ----

/// Converted: `rc2k_core::gfx::pixels::describe`.
pub use rc2k_core::gfx::pixels::describe;

/// On the format at `format`, with the game's table.
pub unsafe fn describe_format(format: *mut u32) -> Result<(), BadInput> {
    if format.is_null() {
        return Err(BadInput("no format"));
    }
    unsafe {
        let numbers: [(u32, u32); NUMBERS] = game_read(NUMBERS_RVA);
        let masks = std::array::from_fn(|n| format.add(1 + n).read_unaligned());
        let (layout, number) = describe(masks, &numbers);
        format.write_unaligned(layout);
        format.add(5).write_unaligned(number);
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { describe_format(r.ebx as *mut u32) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
