//! `physics::ease_view_turn` (0x438B4F): the camera's three turns (`unk_71A688`,
//! `71A690`, `71A68C`) each moved toward its target (`dword_608B50`, `608B58`,
//! `608B54`) by its share (`dword_608BE8`, `608BE4`, `608BEC`; all 1 first when
//! `dword_6EF7CC` is set): the difference `fild`ed, times the share, `fistp`ed, added.
//!
//! In:  nothing
//! Out: eax the last step; the rest kept; no flags read
//!
//! Caller: `sub_43907F`.
//! Self-test: `physics/camera_view.rs` (`view_test_floats`), figures by lot, exact at
//! double precision.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, hook_asm, rva};
use crate::physics::camera_view::{view_regions, view_test_floats};
use crate::physics::flow::{a, base, g, p, x87_run};
use crate::screens::harness::Left;
use crate::service::floats::{Floats, ROUNDED, Tolerance, dispatch_state_floats, floats_entry};
use rc2k_core::physics::camera_view::ease_turns;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::ease_view_turn", rva(0x438B4F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (the exact one, for `physics::camera_avoid_ground`).
pub unsafe fn ease_view_turn(_: &Regs) -> Left {
    if g(0x6EF7CC) != 0 {
        for x in [0x608BE4, 0x608BE8, 0x608BEC] {
            p(x, 0x3F80_0000);
        }
    }
    let mut eax = 0;
    for (target, turn, share) in [
        (0x608B50, 0x71A688, 0x608BE8),
        (0x608B58, 0x71A690, 0x608BE4),
        (0x608B54, 0x71A68C, 0x608BEC),
    ] {
        p(0x62400C, g(target).wrapping_sub(g(turn)));
        x87_run!(
            "fild dword ptr [{b} + 0x22400C]",
            "fmul dword ptr [{s}]",
            "fistp dword ptr [{b} + 0x22400C]";
            b = base(), s = crate::physics::flow::a(share)
        );
        eax = g(0x62400C);
        p(turn, g(turn).wrapping_add(eax));
    }
    Left::nothing().with(EAX, eax)
}

/// The converted version (`rc2k_core::physics::camera_view`, plain floats): the hook's.
// Converted: `rc2k_core::physics::camera_view::ease_turns`.
pub unsafe fn ease_view_turn_plain(_: &Regs) -> Left {
    if g(0x6EF7CC) != 0 {
        for x in [0x608BE4, 0x608BE8, 0x608BEC] {
            p(x, 0x3F80_0000);
        }
    }
    // (target, turn, share), in the original's order.
    const TURNS: [(usize, usize, usize); 3] = [
        (0x608B50, 0x71A688, 0x608BE8),
        (0x608B58, 0x71A690, 0x608BE4),
        (0x608B54, 0x71A68C, 0x608BEC),
    ];
    let mut turns = TURNS.map(|t| g(t.1));
    let step = ease_turns(
        TURNS.map(|t| g(t.0)),
        &mut turns,
        TURNS.map(|t| f32::from_bits(g(t.2))),
    ) as u32;
    p(0x62400C, step);
    for (t, v) in TURNS.iter().zip(turns) {
        p(t.1, v);
    }
    Left::nothing().with(EAX, step)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { ease_view_turn_plain(r) })
}

/// The whole numbers made from floats, held to `ROUNDED` at extended precision.
const FLOATS: [(usize, usize, Tolerance); 2] = [(0x62400C, 4, ROUNDED), (0x71A688, 12, ROUNDED)];

extern "C" fn dispatch(r: &mut Regs) {
    let memory = FLOATS.map(|(x, len, tl)| ((a(x), len), tl));
    let floats = Floats {
        memory: &memory,
        regs: &[(EAX, ROUNDED)],
    };
    dispatch_state_floats(&HOOK, r, call_original, &view_regions(), &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    view_test_floats(
        &HOOK,
        "physics::ease_view_turn",
        call_original,
        &rust,
        &FLOATS,
    )
}
