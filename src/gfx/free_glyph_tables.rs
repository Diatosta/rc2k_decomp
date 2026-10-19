//! `gfx::free_glyph_tables` (0x474D12): `memory::free_list` on `g_glyphTablesMemory`
//! (0x51B984): the fonts' glyph tables given back.
//!
//! In:  nothing
//! Out: ebx = the list; the flags as `free_list` leaves them (carry clear)
//!
//! Caller: `menu::apply_screen_mode` (between `gfx::free_fonts` and `gfx::build_fonts`).
//! The freeing through `part` (in the game `free_list`'s Rust version). Shadow mode runs
//! the original alone (memory is given back). Self-test: a copy whose call goes to a
//! stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::menu::parts::{Part, copy, dispatch_parted, run_both, test_part};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::free_glyph_tables", rva(0x474D12), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LIST_RVA: usize = rva(0x51B984); // g_glyphTablesMemory

/// The Rust version: the registers it leaves.
pub unsafe fn free_glyph_tables(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.ebx = game_addr(LIST_RVA) as u32;
    part(0, &mut r)?;
    Ok(r)
}

/// In the game: `free_list`'s Rust version; its carry is clear.
pub fn game_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::memory::free_list::free_list(r.ebx as *mut crate::memory::MemoryList) }?;
    r.set_carry(false);
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parted(
        &HOOK,
        r,
        call_original,
        &|r, part| unsafe { free_glyph_tables(r, part) },
        game_part,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x474D12,
        0xC,
        &[0x8D, 0x1D, 0x84, 0xB9, 0x51, 0x00],
        &[0x6],
        &[0x4028F5],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    run_both(
        &mut t,
        "once",
        &copy,
        &[],
        &|| {},
        Regs::SENTINEL,
        false,
        &|r| unsafe { free_glyph_tables(r, &mut test_part) },
    );
    t.finish()
}
