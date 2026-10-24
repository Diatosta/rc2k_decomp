//! `hud::draw_stage_bars_1` (0x4B5E29): the stage's progress bars of the split layout,
//! one row a player: as `hud::draw_stage_bar` with a frame of seven quads and each part
//! twice (rows 2 .. 0xE and 0x10 .. 0x1C), the second player's colours from
//! `dword_6D170C` (into `dword_75BFA8`; the first's into `dword_75BFA4`).
//! `hud::draw_stage_bars_2` (0x4B63C2) is the same 0x11E lower.
//!
//! In:  ebp the HUD's record
//! Out: ebx the parts drawn (kept with none), ecx 0; the rest kept
//!
//! Caller: layout 1's drawer (0x4B27F4). The quads through `part` (1 `add_solid_quad`),
//! in the game its Rust version. Refuses more than 0x40 parts and a colour offset past
//! `dword_75BF70`'s four, before anything. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose quads go to a stand-in noting the quad record at each,
//! against the Rust version, parts by lot, at
//! both test control words (the test's values are exact at both).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::rva;
use crate::hud::draw_stage_bar::*;
use crate::hud::store::Slot;

const KINDS_2_RVA: usize = rva(0x6D170C); // dword_6D170C: 0x40
const COLOUR_A_RVA: Slot = Slot::at(0x75BFA4); // dword_75BFA4
const COLOUR_B_RVA: Slot = Slot::at(0x75BFA8); // dword_75BFA8

/// The two players' bars `dy` lower (see the header).
pub unsafe fn bars_with(record: u32, dy: u32) -> Result<(), Refused> {
    unsafe {
        bars(
            record,
            &[KINDS_RVA, KINDS_2_RVA],
            &[COLOUR_A_RVA, COLOUR_B_RVA],
            dy,
        )
    }
}

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::stage_bars` (through `draw_stage_bar::bars`)
pub unsafe fn draw_stage_bars_1(record: u32) -> Result<(), Refused> {
    unsafe { bars_with(record, 0) }
}
