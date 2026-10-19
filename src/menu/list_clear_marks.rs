//! `menu::list_clear_marks` (0x414AC9): the marks (bits 6 and 7) of a list menu's items
//! cleared: each of the `[edi+1]` items (16 bytes from +0x18) its first byte `and 0x3F`,
//! and for an item of kind 2 (low nibble) its `[+1]` choices (2 bytes each from
//! `[+0xC] + 1`) too.
//!
//! In:  edi the list
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! A list of no items goes round 2^32 times, and a kind 2 of no choices 256 times, as
//! the original (the callers' lists have items). Callers: `race::mark_viewer_items`,
//! `sub_4144CF` (three). Shadow mode compares. Self-test: lists of the test's (kinds 2 and
//! others, marks set) against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{check_original, dispatch_regs};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::list_clear_marks", rva(0x414AC9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn list_clear_marks(list: u32) {
    unsafe {
        let mut count = ((list + 1) as *const u8).read() as u32;
        let mut item = (list + 0x18) as *mut u8;
        loop {
            let kind = item.read() & 0x3F;
            item.write(kind);
            if kind & 0xF == 2 {
                let mut n = item.add(1).read();
                let mut choice = ((item.add(0xC) as *const u32).read_unaligned() + 1) as *mut u8;
                loop {
                    choice.write(choice.read() & 0x3F);
                    choice = choice.add(2);
                    n = n.wrapping_sub(1);
                    if n == 0 {
                        break;
                    }
                }
            }
            item = item.add(0x10);
            count = count.wrapping_sub(1);
            if count == 0 {
                break;
            }
        }
    }
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    unsafe { list_clear_marks(r.edi) };
    Ok(*r)
}

/// The list's bytes and its kind 2 items' choices, as far as they go.
pub unsafe fn regions_of(list: u32) -> Vec<Region> {
    unsafe {
        let count = ((list + 1) as *const u8).read() as usize;
        let mut all = vec![(list, 0x18 + 0x10 * count)];
        for n in 0..count {
            let item = list + 0x18 + 0x10 * n as u32;
            if (item as *const u8).read() & 0xF == 2 {
                let choices = ((item + 1) as *const u8).read() as usize;
                let at = ((item + 0xC) as *const u32).read_unaligned();
                let n = if choices == 0 { 0x100 } else { choices };
                all.push((at, 2 * n));
            }
        }
        all
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = unsafe { regions_of(r.edi) };
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut choices = [0xFFu8; 0x40];
    for count in [1u8, 3, 6] {
        let mut list = vec![0xAAu8; 0x18 + 0x10 * count as usize];
        list[1] = count;
        for n in 0..count as usize {
            let item = &mut list[0x18 + 0x10 * n..0x28 + 0x10 * n];
            item[0] = [0xC2, 0x45, 0x82, 0xF1, 0x02, 0x4F][n];
            item[1] = [3, 0, 7, 2, 1, 9][n];
            let at = choices.as_mut_ptr() as u32 + 2 * n as u32;
            item[0xC..0x10].copy_from_slice(&at.to_le_bytes());
        }
        let list_at = list.as_mut_ptr() as u32;
        let regions = unsafe { regions_of(list_at) };
        let first = crate::service::harness::capture(&regions);
        check_original(
            &mut t,
            &format!("{count} items"),
            call_original,
            Regs {
                edi: list_at,
                ..Regs::SENTINEL
            },
            &regions,
            0,
            &|| crate::service::harness::restore(&regions, &first),
            &rust,
        );
    }
    t.finish()
}
