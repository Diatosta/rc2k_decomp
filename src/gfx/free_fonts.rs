//! `gfx::free_fonts` (0x415C55): gives back the memory of the fonts' and sprites' glyph
//! tables.
//!
//! In:  nothing
//! Out: every register preserved
//! `memory::free_list` on `g_glyphTablesMemory` (0x51B984), and `g_fontPalettes`
//! (0x51DEB0) = 0, which is what lets `gfx::build_fonts` build them again.
//! Five callers; `sub_47486C` calls it after the main menu is left.
//! Shadow mode runs the original alone (the memory is given back once) and checks what
//! it left. Self-test: a redirected copy with a list and a variable of the test's.
//! Refuses what `free_list` refuses.

use crate::errors::BadInput;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::build_fonts::PALETTES_RVA;
use crate::gfx::glyph_tables::TABLES_LIST_RVA;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use crate::memory::{MOST_HELD_RVA, MemoryList, OWN_HEAP_RVA};
use crate::redirect::RedirectedCopy;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::free_fonts", rva(0x415C55), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn free_fonts(list: *mut MemoryList, palettes: *mut u32) -> Result<(), BadInput> {
    unsafe {
        free_list(list)?;
        palettes.write_unaligned(0);
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let list = game_addr(TABLES_LIST_RVA) as *mut MemoryList;
    let palettes = game_addr(PALETTES_RVA) as *mut u32;
    if !shadow() {
        if let Err(e) = unsafe { free_fonts(list, palettes) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let before = *r;
    unsafe { call_original(r) };
    let left = unsafe { (list.read_unaligned(), palettes.read_unaligned()) };
    if left.0.head != 0 || left.1 != 0 {
        HOOK.mismatch(|| {
            format!(
                "the original left the list {:x?} and the palettes {:#x}",
                left.0, left.1
            )
        });
    }
    if !r.preserved_except(&before, 0) {
        HOOK.mismatch(|| "the original changed a register".to_string());
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    let list = game_addr(TABLES_LIST_RVA) as *mut MemoryList;
    let palettes = game_addr(PALETTES_RVA) as *mut u32;
    if let Err(e) = unsafe { free_fonts(list, palettes) } {
        return Err(e.into());
    }
    Ok(())
}

// ---- self-test ----

const LEN: usize = 0x18;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8D, 0x1D, 0x84, 0xB9, 0x51, 0x00, 0xE8];
const CALLS: [usize; 1] = [0x07];
const LIST_OPERAND: usize = 0x03;
const PALETTES_OPERAND: usize = 0x0E;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(OWN_HEAP_RVA) } == 1 {
        return t.skip("the game's own heap is in use (altmemenable)");
    }
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x415C55)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            None
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let mut list = MemoryList::default();
    let mut palettes = 0u32;
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let named =
            |operand: usize| u32::from_le_bytes(code[operand..operand + 4].try_into().unwrap());
        if named(LIST_OPERAND) != game_addr(TABLES_LIST_RVA) as u32
            || named(PALETTES_OPERAND) != game_addr(PALETTES_RVA) as u32
        {
            return t.skip("the original is not what the test was made from");
        }
        copy.patch_u32(LIST_OPERAND, &mut list as *mut MemoryList as u32);
        copy.patch_u32(PALETTES_OPERAND, &mut palettes as *mut u32 as u32);
    }
    let figures: [u32; 3] = unsafe { game_read(MOST_HELD_RVA) };
    for sizes in [&[][..], &[0x400u32], &[0x1000, 0x40, 0x2345]] {
        let mut run = |rust: bool| unsafe {
            game_write(MOST_HELD_RVA, [0x9_0000u32, 0x8_0000, 0xA_0000]);
            list = MemoryList {
                head: 0,
                total: 0x30,
                cursor: 0xAAAA_AAAA,
                remaining: 0xAAAA_AAAA,
            };
            palettes = 0x1234_5678;
            let taken = sizes
                .iter()
                .filter(|&&bytes| allocate(&mut list, bytes).is_ok())
                .count();
            let before = Regs::SENTINEL;
            let ok = if rust {
                free_fonts(&mut list, &mut palettes).is_ok()
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
            };
            let figures_now: [u32; 3] = game_read(MOST_HELD_RVA);
            (ok, taken == sizes.len(), list, palettes, figures_now)
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.1, || {
            format!("sizes {sizes:x?}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    unsafe { game_write(MOST_HELD_RVA, figures) };
    t.finish()
}
