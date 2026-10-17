//! `memory::free_list` (0x4028F5): gives back every block of a list.
//!
//! In:  ebx = the list (`MemoryList`), or 0
//! Out: every register preserved; CF clear (read by `sub_4D06F3`'s `jb`)
//! Nothing happens without a list. The bytes held now are counted down by the list's total
//! (also when it has no block). Then, if it has one: the list is emptied (head, total, and
//! through `sub_402941` the cursor and what remains, all 0) and each block is given back
//! with `VirtualFree`, or, with the setting `altmemenable`, to the game's own heap
//! (`sub_402A92`, not ported).
//! 38 callers. Shadow mode runs the original alone and checks that it left the list empty.
//! Manual patch with a private copy (0x4C bytes, three calls relocated): a conditional
//! jump sits in the first bytes.
//! Refuses the game's own heap when the list has a block.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::memory::allocate::allocate;
use crate::memory::{HELD_RVA, MOST_HELD_RVA, MemoryList, OWN_HEAP_RVA};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::Memory::{
    MEM_FREE, MEM_RELEASE, MEMORY_BASIC_INFORMATION, VirtualFree, VirtualQuery,
};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("memory::free_list", rva(0x4028F5), thunk, &ORIGINAL)
    .private_copy(0x4C)
    .relocate_calls(&[0x22, 0x38, 0x3F])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn free_list(list: *mut MemoryList) -> Result<(), BadInput> {
    unsafe {
        if list.is_null() {
            return Ok(());
        }
        let mut block = (*list).head;
        if block != 0 && game_read::<u32>(OWN_HEAP_RVA) == 1 {
            return Err(BadInput("the game's own heap is in use (altmemenable)"));
        }
        game_write(
            HELD_RVA,
            game_read::<u32>(HELD_RVA).wrapping_sub((*list).total),
        );
        if block == 0 {
            return Ok(());
        }
        *list = MemoryList::default();
        while block != 0 {
            let before = (block as *const u32).read();
            VirtualFree(block as _, 0, MEM_RELEASE);
            block = before;
        }
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let list = r.ebx as *mut MemoryList;
    if !shadow() {
        match unsafe { free_list(list) } {
            // The original ends in a `test`: the carry is clear (`marks::tidy_marks`'
            // `sub_4D06F3` reads it with `jb`).
            Ok(()) => r.set_carry(false),
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }
    let had_blocks = !list.is_null() && unsafe { (*list).head } != 0;
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let emptied = !had_blocks || unsafe { list.read_unaligned() } == MemoryList::default();
    if !emptied || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!("the original left the list {:x?}", unsafe {
                list.read_unaligned()
            })
        });
    }
}

// ---- self-test: lists of blocks taken for the purpose, given back by both; the memory
// figures are put back ----

/// Whether the address is in memory that has been given back.
fn is_free(address: u32) -> bool {
    let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let got = unsafe { VirtualQuery(address as _, &mut info, std::mem::size_of_val(&info)) };
    got != 0 && info.State == MEM_FREE
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(OWN_HEAP_RVA) } == 1 {
        return t.skip("the game's own heap is in use (altmemenable)");
    }
    let figures: [u32; 3] = unsafe { game_read(MOST_HELD_RVA) };
    for sizes in [&[][..], &[1u32], &[0x1000, 0x40, 0x12345], &[0, 0, 7]] {
        let run = |rust: bool| {
            unsafe { game_write(MOST_HELD_RVA, [0x9_0000u32, 0x8_0000, 0xA_0000]) };
            let mut list = MemoryList {
                head: 0,
                total: 0x30,
                cursor: 0xAAAA_AAAA,
                remaining: 0xAAAA_AAAA,
            };
            let blocks: Vec<u32> = sizes
                .iter()
                .filter_map(|&bytes| unsafe { allocate(&mut list, bytes) }.ok())
                .map(|memory| memory as u32)
                .collect();
            let before = Regs {
                ebx: &mut list as *mut MemoryList as u32,
                ..Regs::SENTINEL
            };
            let ok = if rust {
                unsafe { free_list(&mut list) }.is_ok()
            } else {
                let mut ro = before;
                unsafe { call_original(&mut ro) };
                ro.preserved_except(&before, 0)
            };
            let all_free = blocks.iter().all(|&memory| is_free(memory));
            let figures_now: [u32; 3] = unsafe { game_read(MOST_HELD_RVA) };
            (ok, blocks.len() == sizes.len(), all_free, list, figures_now)
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.1 && theirs.2, || {
            format!("sizes {sizes:x?}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    // No list at all.
    let before = Regs {
        ebx: 0,
        ..Regs::SENTINEL
    };
    let held: u32 = unsafe { game_read(HELD_RVA) };
    let mut ro = before;
    unsafe { call_original(&mut ro) };
    let mine = unsafe { free_list(std::ptr::null_mut()) };
    t.check(
        mine.is_ok()
            && ro.preserved_except(&before, 0)
            && unsafe { game_read::<u32>(HELD_RVA) } == held,
        || "no list: something changed".to_string(),
    );
    unsafe { game_write(MOST_HELD_RVA, figures) };
    t.finish()
}
