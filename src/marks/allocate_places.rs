//! `marks::allocate_places` (0x4C38B1): `init_marks`' step 6, the places' block taken and started.
//!
//! In:  nothing
//! Out: eax, esi the block, ebx the list
//!
//! See `marks/blocks.rs`. One caller, `marks::init_marks`. Shadow mode runs the original alone (memory
//! cannot be taken twice). Self-test: a redirected copy whose calls go to stand-ins (the
//! block a buffer of the test's), the game's variables put back afterwards.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::marks::blocks::*;
use crate::race::level_init::GIVES;
use crate::race::mud::{copy, run_both};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::allocate_places", rva(0x4C38B1), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { allocate_places(r, &mut game_part) } {
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
    let entry: &[u8] = &[0xB8, 0x24, 0x70, 0x00, 0x00];
    let copy = match copy(0x4C38B1, 0x44, entry, &[0xB], &[(0x4028A7, 0)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(give);
    let block = vec![0xAAu8; PLACES_LEN as usize];
    let mut regions: Vec<Region> = [PLACES_RVA].map(at).to_vec();
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
        &|r| unsafe { allocate_places(r, &mut test_give) },
    );
    regions.pop();
    restore(&regions, &saved);
    t.finish()
}

fn extra_regions() -> Vec<Region> {
    Vec::new()
}

fn prepare() {}
