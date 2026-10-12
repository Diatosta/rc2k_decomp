//! `saves::free_best_times` (0x47C627): gives back the memory of the best times' tables.
//!
//! In:  nothing
//! Out: every register preserved
//! `memory::free_list` on `g_bestTimesMemory` (0x7475D0), and `g_bestTimesAllocated`
//! (0x747610) = 0. The tables' addresses stay in their variables.
//! One caller, the main loop, after `saves::save_game_data`.
//! Shadow mode runs the original alone (the memory is given back once) and checks what
//! it left. Self-test: a redirected copy with a list and a variable of the test's.
//! Refuses what `free_list` refuses.

use crate::errors::BadInput;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use crate::memory::{MOST_HELD_RVA, MemoryList, OWN_HEAP_RVA};
use crate::redirect::RedirectedCopy;
use crate::saves::allocate_best_times::VARS_RVA as ALLOCATED_RVA;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("saves::free_best_times", rva(0x47C627), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const TABLES_LIST_RVA: usize = rva(0x7475D0); // g_bestTimesMemory

// ---- Rust version ----

pub unsafe fn free_best_times(list: *mut MemoryList, allocated: *mut u32) -> Result<(), BadInput> {
    unsafe {
        free_list(list)?;
        allocated.write_unaligned(0);
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let list = game_addr(TABLES_LIST_RVA) as *mut MemoryList;
    let allocated = game_addr(ALLOCATED_RVA) as *mut u32;
    if !shadow() {
        if let Err(e) = unsafe { free_best_times(list, allocated) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let before = *r;
    unsafe { call_original(r) };
    let left = unsafe { (list.read_unaligned(), allocated.read_unaligned()) };
    if left.0.head != 0 || left.1 != 0 {
        HOOK.mismatch(|| {
            format!(
                "the original left the list {:x?} and the allocated {:#x}",
                left.0, left.1
            )
        });
    }
    if !r.preserved_except(&before, 0) {
        HOOK.mismatch(|| "the original changed a register".to_string());
    }
}

// ---- self-test ----

const LEN: usize = 0x18;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8D, 0x1D, 0xD0, 0x75, 0x74, 0x00, 0xE8];
const CALLS: [usize; 1] = [0x07];
const LIST_OPERAND: usize = 0x03;
const ALLOCATED_OPERAND: usize = 0x0E;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(OWN_HEAP_RVA) } == 1 {
        return t.skip("the game's own heap is in use (altmemenable)");
    }
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x47C627)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            None
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let mut list = MemoryList::default();
    let mut allocated = 0u32;
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let named =
            |operand: usize| u32::from_le_bytes(code[operand..operand + 4].try_into().unwrap());
        if named(LIST_OPERAND) != game_addr(TABLES_LIST_RVA) as u32
            || named(ALLOCATED_OPERAND) != game_addr(ALLOCATED_RVA) as u32
        {
            return t.skip("the original is not what the test was made from");
        }
        copy.patch_u32(LIST_OPERAND, &mut list as *mut MemoryList as u32);
        copy.patch_u32(ALLOCATED_OPERAND, &mut allocated as *mut u32 as u32);
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
            allocated = 0x1234_5678;
            let taken = sizes
                .iter()
                .filter(|&&bytes| allocate(&mut list, bytes).is_ok())
                .count();
            let before = Regs::SENTINEL;
            let ok = if rust {
                free_best_times(&mut list, &mut allocated).is_ok()
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
            };
            let figures_now: [u32; 3] = game_read(MOST_HELD_RVA);
            (ok, taken == sizes.len(), list, allocated, figures_now)
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.1, || {
            format!("sizes {sizes:x?}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    unsafe { game_write(MOST_HELD_RVA, figures) };
    t.finish()
}
