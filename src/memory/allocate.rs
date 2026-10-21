//! `memory::allocate` (0x4028A7): takes a block of memory and puts it on a list.
//!
//! In:  eax = bytes, ebx = the list (`MemoryList`)
//! Out: eax = the block's memory, zeroed; carry clear; everything else preserved
//! The block is 0x40 bytes larger than asked for: a header in front, which holds the block
//! taken before on the same list and the bytes asked for. It comes from `VirtualAlloc`
//! (committed, read and write; none is fatal error 0x80108571 "Need more virtual memory!"),
//! or, with the setting `altmemenable`, from a heap of the game's own (`sub_402A05`, not
//! ported). The list's total and the three memory figures are counted up.
//! 56 callers. Shadow mode runs the original alone (memory cannot be taken twice) and
//! checks the block and the list it leaves.
//! Refuses the game's own heap, a null list and a size that with the header wraps to 0.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow,
};
use crate::memory::{HEADER_LEN, MOST_HELD_RVA, MemoryList, OWN_HEAP_RVA, with_figures};
use rc2k_core::memory::block_size;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::Memory::{MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE, VirtualAlloc};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("memory::allocate", rva(0x4028A7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The block's memory.
pub unsafe fn allocate(list: *mut MemoryList, bytes: u32) -> Result<*mut u8, Refused> {
    unsafe {
        if game_read::<u32>(OWN_HEAP_RVA) == 1 {
            return Err(BadInput("the game's own heap is in use (altmemenable)").into());
        }
        // Converted: `rc2k_core::memory` (`block_size`, `List::add`, `Figures::taken`).
        let whole = block_size(bytes);
        let (false, Some(whole)) = (list.is_null(), whole) else {
            return Err(BadInput("no list, or a size that wraps").into());
        };
        let block = VirtualAlloc(
            std::ptr::null(),
            whole as usize,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        ) as *mut u32;
        if block.is_null() {
            return Err(FatalError {
                code: 0x8010_8571,
                message: "Need more virtual memory!",
            }
            .into());
        }
        let header = (*list).add(block as u32, bytes);
        block.write(header[0]);
        block.add(1).write(header[1]);
        with_figures(|f| f.taken(bytes));
        Ok((block as *mut u8).add(HEADER_LEN as usize))
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (list, bytes) = (r.ebx as *mut MemoryList, r.eax);
    if !shadow() {
        match unsafe { allocate(list, bytes) } {
            Ok(memory) => {
                r.eax = memory as u32;
                r.set_carry(false);
            }
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }
    let before = (!list.is_null()).then(|| unsafe { list.read_unaligned() });
    let mut o = *r;
    unsafe { call_original(&mut o) };
    // As the Rust version leaves things: the block on the list, its header, zeroed memory.
    let as_rust = match before {
        Some(before) if !o.carry() => unsafe {
            let after = list.read_unaligned();
            let block = o.eax.wrapping_sub(HEADER_LEN) as *const u32;
            after.head == block as u32
                && after.total == before.total.wrapping_add(bytes)
                && block.read() == before.head
                && block.add(1).read() == bytes
                && (bytes == 0 || (o.eax as *const u8).read() == 0)
        },
        _ => true,
    };
    if !as_rust || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "bytes={bytes:#x} list={:#x}: the original returned {:#x} carry={} and left the list {:x?} (before {before:x?})",
                r.ebx,
                o.eax,
                o.carry(),
                (!list.is_null()).then(|| unsafe { list.read_unaligned() })
            )
        });
    }
    r.eax = o.eax;
    r.eflags = o.eflags;
}

// ---- self-test: blocks taken on lists of the test's, by both, and given back; the memory
// figures are put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(OWN_HEAP_RVA) } == 1 {
        return t.skip("the game's own heap is in use (altmemenable)");
    }
    let figures: [u32; 3] = unsafe { game_read(MOST_HELD_RVA) };
    for sizes in [
        &[1u32][..],
        &[0],
        &[0x1000, 0x40, 0x12345],
        &[0xFC0, 0xFC1, 7],
    ] {
        // What a side leaves, without the addresses, which are its own.
        let run = |rust: bool| {
            unsafe { game_write(MOST_HELD_RVA, [0x5000u32, 0x4000, 0x9000]) };
            let mut list = MemoryList {
                head: 0,
                total: 0x10,
                cursor: 0xAAAA_AAAA,
                remaining: 0xAAAA_AAAA,
            };
            let mut seen = Vec::new();
            for &bytes in sizes {
                let previous = list.head;
                let before = Regs {
                    eax: bytes,
                    ebx: &mut list as *mut MemoryList as u32,
                    ..Regs::SENTINEL
                };
                let memory = if rust {
                    unsafe { allocate(&mut list, bytes) }.map_or(0, |memory| memory as u32)
                } else {
                    let mut ro = before;
                    unsafe { call_original(&mut ro) };
                    if ro.carry() || !ro.preserved_except(&before, EAX) {
                        0
                    } else {
                        ro.eax
                    }
                };
                if memory == 0 {
                    seen.push(vec![u32::MAX]);
                    break;
                }
                let block = memory.wrapping_sub(HEADER_LEN) as *const u32;
                let zeroed =
                    unsafe { std::slice::from_raw_parts(memory as *const u8, bytes as usize) }
                        .iter()
                        .all(|&byte| byte == 0);
                seen.push(unsafe {
                    vec![
                        (list.head == block as u32) as u32,
                        (block.read() == previous) as u32,
                        block.add(1).read(),
                        zeroed as u32,
                        list.total,
                        list.cursor,
                    ]
                });
            }
            let figures_now: [u32; 3] = unsafe { game_read(MOST_HELD_RVA) };
            // Given back with the Rust version of the other half, whichever side took it.
            let _ = unsafe { crate::memory::free_list::free_list(&mut list) };
            (seen, figures_now)
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(
            theirs == mine && !theirs.0.iter().any(|block| block == &[u32::MAX]),
            || format!("sizes {sizes:x?}: original {theirs:x?}; rust {mine:x?}"),
        );
    }
    unsafe { game_write(MOST_HELD_RVA, figures) };
    t.finish()
}
