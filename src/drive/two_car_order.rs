//! `drive::two_car_order` (0x460CDA): the two cars' places in a two-car race (the car
//! blocks' +0xFA4, `g_carBlockStore`): -1 for both outside mode 0 (`dword_62389C`) or with
//! one car (`dword_63DB94` 1); left as they are while either has +0xA8 set; else 1 for the
//! one ahead (checkpoint +0x94, then +0x624, each by the sign of the `cmp`), 2 for the
//! other.
//!
//! In:  nothing
//! Out: ebx the last figure compared; the rest kept; no flags read
//!
//! Caller: `drive::replay_step`.
//! Self-test: the original and the Rust version on the two blocks' figures (saved and put
//! back): modes, ahead, behind, level.

use crate::errors::Refused;
use crate::framework::{EBX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::two_car_order", rva(0x460CDA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const B: usize = 0x70F3EC; // g_carBlockStore
const NEXT: usize = 0xFE0;

/// The Rust version.
pub fn two_car_order(entry: &Regs) -> Left {
    let mut ebx = entry.ebx;
    let set = |x: u32, y: u32| {
        p(B + 0xFA4, x);
        p(B + NEXT + 0xFA4, y);
    };
    if g(0x62389C) != 0 || g(0x63DB94) == 1 {
        set(u32::MAX, u32::MAX);
    } else if g(B + 0xA8) == 0 && g(B + NEXT + 0xA8) == 0 {
        ebx = g(B + 0x94);
        let d = ebx.wrapping_sub(g(B + NEXT + 0x94));
        let behind = if (d as i32) < 0 {
            true
        } else if d != 0 {
            false
        } else {
            ebx = g(B + 0x624);
            (ebx.wrapping_sub(g(B + NEXT + 0x624)) as i32) < 0
        };
        if behind {
            set(2, 1);
        } else {
            set(1, 2);
        }
    }
    Left::nothing().with(EBX, ebx)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(two_car_order(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(a(B + 0xFA4), 4), (a(B + NEXT + 0xFA4), 4)],
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: Vec<Region> = vec![(a(0x70F3EC), 0x1FC0), (a(0x62389C), 4), (a(0x63DB94), 4)];
    let saved = capture(&all);
    for case in 0..24u32 {
        let set = || {
            p(0x62389C, if case % 6 == 5 { 2 } else { 0 });
            p(0x63DB94, if case % 7 == 6 { 1 } else { 2 });
            p(0x70F3EC + 0xA8, (case % 8 == 7) as u32);
            p(0x70F3EC + 0xFE0 + 0xA8, 0);
            p(0x70F3EC + 0x94, [3u32, 5, 4, 4][(case % 4) as usize]);
            p(0x70F3EC + 0xFE0 + 0x94, 4);
            p(0x70F3EC + 0x624, case % 3);
            p(0x70F3EC + 0xFE0 + 0x624, 1);
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
