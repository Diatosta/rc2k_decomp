//! `service::load_setup_values` (0x47070C): copies three setup values of the current car into
//! the service area's edit variables, unless `dword_75AAE0` is 1.
//!
//! In:  nothing
//! Out: every register preserved
//! Each value goes into a pair of variables: car +0x10C into 0x75ACA0 and 0x75AC9C, +0x120
//! into 0x75AC94 and 0x75AC90, +0x118 into 0x75ACB8 and 0x75ACB4. A fourth pair is treated
//! differently: the function loads car +0x114 but then stores 0x75ACAC, which it does not
//! change, into 0x75ACA8. The names are a reading: what the three values are (the key handlers
//! step 0x75AC9C between 0 and 5) has not been worked out.
//! It is not called directly: its address is in the service area's list at 0x759436.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::service::state::current_player;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::load_setup_values",
    rva(0x47070C),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const SKIP_RVA: usize = rva(0x75AAE0); // dword_75AAE0: 1 = leave the edit variables alone

/// The eight edit variables, 0x75AC90 .. 0x75ACB8, in address order.
const EDIT_RVAS: [usize; 8] = [
    rva(0x75AC90),
    rva(0x75AC94),
    rva(0x75AC9C),
    rva(0x75ACA0),
    rva(0x75ACA8),
    rva(0x75ACAC),
    rva(0x75ACB4),
    rva(0x75ACB8),
];

fn edit_values() -> [u32; 8] {
    EDIT_RVAS.map(|at| unsafe { game_read(at) })
}

fn set_edit_values(values: &[u32; 8]) {
    for (at, v) in EDIT_RVAS.iter().zip(values) {
        unsafe { game_write(*at, *v) };
    }
}

// ---- Rust version ----

/// Refuses a missing player: the original reads through a null pointer.
pub unsafe fn load_setup_values() -> Result<(), BadInput> {
    unsafe {
        if game_read::<u32>(SKIP_RVA) == 1 {
            return Ok(());
        }
        let Some(player) = current_player() else {
            return Err(BadInput(
                "no current player object: the original reads through null",
            ));
        };
        let field = |at: u32| (player.wrapping_add(at) as *const u32).read_unaligned();
        for (from, first, second) in [
            (0x10C, 0x75ACA0, 0x75AC9C),
            (0x120, 0x75AC94, 0x75AC90),
            (0x118, 0x75ACB8, 0x75ACB4),
        ] {
            let value = field(from);
            game_write(rva(first), value);
            game_write(rva(second), value);
        }
        // The original loads car +0x114 here and then stores this instead.
        let kept: u32 = game_read(rva(0x75ACAC));
        game_write(rva(0x75ACA8), kept);
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();

    if !shadow() {
        if unsafe { load_setup_values() }.is_err() {
            unsafe { call_original(r) };
        }
        return;
    }

    // Shadow: the function only copies, so running it twice changes nothing.
    if unsafe { load_setup_values() }.is_err() {
        unsafe { call_original(r) };
        return;
    }
    let mine = edit_values();
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = edit_values();
    if theirs != mine || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| format!("original {theirs:x?} rust {mine:x?}"));
    }
}

// ---- self-test: both settings of the skip flag, on the current car ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if current_player().is_none() {
        return t.skip("there is no current player object");
    }
    let saved_values = edit_values();
    let saved_skip: u32 = unsafe { game_read(SKIP_RVA) };
    let stale: [u32; 8] = std::array::from_fn(|i| 0x5EED_0000 + i as u32);

    for skip in [0u32, 1, 2] {
        unsafe { game_write(SKIP_RVA, skip) };
        set_edit_values(&stale);
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let theirs = edit_values();

        set_edit_values(&stale);
        let result = unsafe { load_setup_values() };
        let mine = edit_values();

        let preserved = ro.preserved_except(&before, 0);
        t.check(result.is_ok() && theirs == mine && preserved, || {
            format!(
                "skip={skip}: original {theirs:x?} preserved={preserved}; rust {result:?} {mine:x?}"
            )
        });
    }

    unsafe { game_write(SKIP_RVA, saved_skip) };
    set_edit_values(&saved_values);
    t.finish()
}
