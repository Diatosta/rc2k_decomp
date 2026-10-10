//! `menu::update_blink` (0x47E826): once a frame, works out from the clock what blinks and
//! whether the game has been left alone for a long time.
//!
//! In:  nothing
//! Out: every register preserved
//! After `update_frame_clock`:
//!   - `dword_7559A4` (the highlight's blink, as the service area's markers use it) is 1 while
//!     bit 8 of `clock - dword_7556C4` is clear, so it is on for 256 ms and off for 256 ms,
//!     counted from the last key; it stays 0 while `dword_7556AC` is 1;
//!   - `dword_7559A8` (the text entry's cursor) is 1 while bit 9 of `clock - dword_7556C8` is
//!     clear: 512 ms on, 512 ms off;
//!   - `dword_755551` is 1 once `clock - dword_7556C4` has reached 300000: five minutes
//!     without a key.
//!
//! 179 callers. Shadow mode lets the original run and checks what it stored against the Rust
//! arithmetic on the clock value it used.
//! Manual patch with a private copy (0x95 bytes, one call relocated): a call sits in the
//! first bytes.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::menu::update_frame_clock::{CLOCK_RVA, update_frame_clock};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::update_blink", rva(0x47E826), thunk, &ORIGINAL)
    .private_copy(0x95)
    .relocate_calls(&[1])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const LAST_KEY_TIME_RVA: usize = rva(0x7556C4); // dword_7556C4: the clock at the last key
const CURSOR_TIME_RVA: usize = rva(0x7556C8); // dword_7556C8: what the cursor's blink counts from
const BLINK_OFF_RVA: usize = rva(0x7556AC); // dword_7556AC: 1 = the highlight does not blink
const BLINK_RVA: usize = rva(0x7559A4); // dword_7559A4
const CURSOR_ON_RVA: usize = rva(0x7559A8); // dword_7559A8
const IDLE_RVA: usize = rva(0x755551); // dword_755551
const IDLE_AFTER_MS: i32 = 300_000;

// ---- Rust version ----

/// (highlight blink, cursor on, idle) for a clock value.
pub fn blink_states(
    clock: u32,
    last_key: u32,
    cursor_time: u32,
    blink_off: u32,
) -> (u32, u32, u32) {
    let since_key = clock.wrapping_sub(last_key);
    let blink = (blink_off != 1 && since_key & 0x100 == 0) as u32;
    let cursor_on = (clock.wrapping_sub(cursor_time) & 0x200 == 0) as u32;
    let idle = (since_key as i32 >= IDLE_AFTER_MS) as u32;
    (blink, cursor_on, idle)
}

/// What the three variables should be for the clock the game holds now.
unsafe fn states_now() -> (u32, u32, u32) {
    unsafe {
        blink_states(
            game_read(CLOCK_RVA),
            game_read(LAST_KEY_TIME_RVA),
            game_read(CURSOR_TIME_RVA),
            game_read(BLINK_OFF_RVA),
        )
    }
}

unsafe fn stored() -> (u32, u32, u32) {
    unsafe {
        (
            game_read(BLINK_RVA),
            game_read(CURSOR_ON_RVA),
            game_read(IDLE_RVA),
        )
    }
}

pub fn update_blink() {
    update_frame_clock();
    unsafe {
        let (blink, cursor_on, idle) = states_now();
        game_write(BLINK_RVA, blink);
        game_write(CURSOR_ON_RVA, cursor_on);
        game_write(IDLE_RVA, idle);
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        update_blink();
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let (theirs, mine) = unsafe { (stored(), states_now()) };
    if theirs != mine || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!(
                "clock={} last_key={} cursor_time={}: original {theirs:?} rust {mine:?}",
                unsafe { game_read::<u32>(CLOCK_RVA) },
                unsafe { game_read::<u32>(LAST_KEY_TIME_RVA) },
                unsafe { game_read::<u32>(CURSOR_TIME_RVA) }
            )
        });
    }
}

// ---- self-test: times since the last key on both sides of every boundary ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [
        at(CLOCK_RVA),
        at(LAST_KEY_TIME_RVA),
        at(CURSOR_TIME_RVA),
        at(BLINK_OFF_RVA),
        at(BLINK_RVA),
        at(CURSOR_ON_RVA),
        at(IDLE_RVA),
    ];
    let saved = capture(&regions);
    // The real clock is read by both versions, so each is checked against the arithmetic on
    // the clock value it stored. Distances are kept well inside their half-periods, so a few
    // milliseconds between the runs do not move them across a boundary... except by intent.
    for since_key in [50u32, 300, 600, 299_000, 300_500, 1_000_000] {
        for since_cursor in [100u32, 700, 1100] {
            for blink_off in [0u32, 1, 2] {
                let run = |f: &dyn Fn()| unsafe {
                    let now = crate::system::tick_count::tick_count();
                    game_write(LAST_KEY_TIME_RVA, now.wrapping_sub(since_key));
                    game_write(CURSOR_TIME_RVA, now.wrapping_sub(since_cursor));
                    game_write(BLINK_OFF_RVA, blink_off);
                    for rva in [BLINK_RVA, CURSOR_ON_RVA, IDLE_RVA] {
                        game_write(rva, 0x5EEDu32);
                    }
                    f();
                    (stored(), states_now())
                };
                let before = Regs::SENTINEL;
                let preserved = std::cell::Cell::new(true);
                let (stored_o, wanted_o) = run(&|| {
                    let mut ro = before;
                    unsafe { call_original(&mut ro) };
                    preserved.set(ro.preserved_except(&before, 0));
                });
                let (stored_r, wanted_r) = run(&update_blink);
                t.check(stored_o == wanted_o && stored_r == wanted_r && preserved.get(), || {
                    format!(
                        "since_key={since_key} since_cursor={since_cursor} blink_off={blink_off}: original {stored_o:?} (arithmetic says {wanted_o:?}) preserved={}; rust {stored_r:?} ({wanted_r:?})",
                        preserved.get()
                    )
                });
            }
        }
    }
    restore(&regions, &saved);
    t.finish()
}
