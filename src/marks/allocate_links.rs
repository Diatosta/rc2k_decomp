//! `marks::allocate_links` (0x4C5398): in `init_marks`' step 3, the links' block taken and linked, the corners scaled.
//!
//! In:  nothing
//! Out: as `marks::scale_corners` leaves them, ecx 0
//!
//! See `marks/blocks.rs`. One caller, `sub_4C3B01`. Shadow mode runs the original alone (memory
//! cannot be taken twice). Self-test: a redirected copy whose calls go to stand-ins (the
//! block a buffer of the test's), the game's variables put back afterwards.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::marks::blocks::*;
use crate::race::level_init::GIVES;
use crate::race::mud::{copy, run_both};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::allocate_links", rva(0x4C5398), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { allocate_links(r, &mut game_part) } {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let entry: &[u8] = &[0xB8, 0x20, 0x10, 0x07, 0x00];
    let copy = match copy(
        0x4C5398,
        0x71,
        entry,
        &[0xB, 0x6B],
        &[(0x4028A7, 0), (0x4C6420, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(give);
    let block = vec![0xAAu8; LINKS_LEN as usize];
    let mut regions: Vec<Region> = [LINKS_RVA].map(at).to_vec();
    regions.extend(extra_regions());
    let saved = capture(&regions);
    regions.push((block.as_ptr() as u32, block.len()));
    run_both(
        &mut t,
        "a block of the test's",
        &copy,
        &regions,
        &|| unsafe {
            std::ptr::write_bytes(block.as_ptr() as *mut u8, 0xAA, block.len());
            GIVES.store(block.as_ptr() as u32, Ordering::SeqCst);
            prepare();
        },
        &|r| unsafe { allocate_links(r, &mut test_give) },
    );
    regions.pop();
    restore(&regions, &saved);
    t.finish()
}

fn extra_regions() -> Vec<Region> {
    let mut r = corner_regions();
    r.push((game_addr(CORNERS_RVA) as u32, 0x80 * CORNERS as usize));
    r
}

fn prepare() {
    crate::marks::scale_corners::fill_corners(7);
}
