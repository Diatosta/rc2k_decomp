//! `menu::update_frame_clock` (0x411019): stores `tick_count` in `dword_51899C`, the clock the
//! menu code measures blinking and idle time against.
//!
//! In:  nothing
//! Out: eax = the clock; everything else preserved
//! Three callers. The clock moves, so shadow mode and the self-test accept any value between
//! the Rust answers from before and after the original.
//! Manual patch with a private copy (0x0B bytes, the call relocated): the function starts
//! with a call.

use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow,
};
use crate::system::tick_count::tick_count;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::update_frame_clock", rva(0x411019), thunk, &ORIGINAL)
        .private_copy(0x0B)
        .relocate_calls(&[0])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const CLOCK_RVA: usize = rva(0x51899C); // dword_51899C

// ---- Rust version ----

pub fn update_frame_clock() -> u32 {
    let now = tick_count();
    unsafe { game_write(CLOCK_RVA, now) };
    now
}

// ---- hook ----

fn between(first: u32, value: u32, last: u32) -> bool {
    value.wrapping_sub(first) <= last.wrapping_sub(first)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let first = update_frame_clock();
    if !shadow() {
        r.eax = first;
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let stored: u32 = unsafe { game_read(CLOCK_RVA) };
    let last = tick_count();
    if !between(first, o.eax, last) || stored != o.eax || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "original {} stored {stored}; rust {first} then {last}",
                o.eax
            )
        });
    }
    r.eax = o.eax;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved: u32 = unsafe { game_read(CLOCK_RVA) };
    for _ in 0..3 {
        let first = update_frame_clock();
        let stored_r: u32 = unsafe { game_read(CLOCK_RVA) };
        unsafe { game_write(CLOCK_RVA, 0x5EED_0001u32) };
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let stored_o: u32 = unsafe { game_read(CLOCK_RVA) };
        let last = tick_count();
        let ok = between(first, ro.eax, last)
            && stored_o == ro.eax
            && stored_r == first
            && ro.preserved_except(&before, EAX);
        t.check(ok, || {
            format!(
                "original {} stored {stored_o}; rust {first} stored {stored_r}, then {last}",
                ro.eax
            )
        });
    }
    unsafe { game_write(CLOCK_RVA, saved) };
    t.finish()
}
