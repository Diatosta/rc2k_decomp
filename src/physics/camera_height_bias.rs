//! `physics::camera_height_bias` (0x43852D): with the mode's record's +0x54 not 0:
//! `flt_60631C` / `flt_606290` held to 0 ..= 1 (1 less it for +0x54 below 0), x
//! `flt_5C5D98` x `flt_60628C` (`fistp`) into `dword_606288` and added to the camera's
//! height `unk_71A680`.
//!
//! In:  nothing
//! Out: esi the mode's record; eax the height added when it ran; the rest kept; no
//!      flags read
//!
//! Caller: `sub_437A53`.
//! Self-test: `physics/camera_view.rs` (`view_test_floats`), modes and figures by lot,
//! exact at double precision.

use crate::errors::Refused;
use crate::framework::{EAX, ESI, RawHook, Regs, hook_asm, rva};
use crate::physics::camera_view::{rd, record, record_plain, view_regions, view_test_floats};
use crate::physics::flow::{a, base, g, hold_high, hold_low, p, x87_run};
use crate::screens::harness::Left;
use crate::service::floats::{Floats, ROUNDED, Tolerance, dispatch_state_floats, floats_entry};
use rc2k_core::physics::camera_view::height_bias;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::camera_height_bias",
    rva(0x43852D),
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

/// The Rust version (the exact one, for `physics::update_camera`).
pub unsafe fn camera_height_bias(_: &Regs) -> Left {
    let esi = record();
    let left = Left::nothing().with(ESI, esi);
    if rd(esi + 0x54) == 0 {
        return left;
    }
    x87_run!(
        "fld dword ptr [{b} + 0x20631C]",
        "fdiv dword ptr [{b} + 0x206290]",
        "fstp dword ptr [{b} + 0x22400C]";
        b = base()
    );
    hold_low(0x62400C, 0x5C5D90);
    hold_high(0x62400C, 0x5C5D38);
    if (rd(esi + 0x54) as i32) < 0 {
        x87_run!(
            ".byte 0xD9, 0xE8", // fld1
            "fsub dword ptr [{b} + 0x22400C]",
            "fstp dword ptr [{b} + 0x22400C]";
            b = base()
        );
    }
    x87_run!(
        "fld dword ptr [{b} + 0x22400C]",
        "fmul dword ptr [{b} + 0x1C5D98]",
        "fmul dword ptr [{b} + 0x20628C]",
        "fistp dword ptr [{b} + 0x22400C]";
        b = base()
    );
    let eax = g(0x62400C);
    p(0x606288, eax);
    p(0x71A680, g(0x71A680).wrapping_add(eax));
    left.with(EAX, eax)
}

/// The converted version (`rc2k_core::physics::camera_view`, plain floats): the hook's.
// Converted: `rc2k_core::physics::camera_view::height_bias`.
pub unsafe fn camera_height_bias_plain(_: &Regs) -> Left {
    let esi = record_plain();
    let left = Left::nothing().with(ESI, esi);
    if rd(esi + 0x54) == 0 {
        return left;
    }
    let f = |x: usize| f32::from_bits(g(x));
    let n = height_bias(
        f(0x60631C),
        f(0x606290),
        f(0x5C5D90),
        f(0x5C5D38),
        (rd(esi + 0x54) as i32) < 0,
        f(0x5C5D98),
        f(0x60628C),
    ) as u32;
    p(0x62400C, n);
    p(0x606288, n);
    p(0x71A680, g(0x71A680).wrapping_add(n));
    left.with(EAX, n)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { camera_height_bias_plain(r) })
}

/// The whole numbers made from floats, held to `ROUNDED` at extended precision.
const FLOATS: [(usize, usize, Tolerance); 3] = [
    (0x62400C, 4, ROUNDED),
    (0x606288, 4, ROUNDED),
    (0x71A680, 4, ROUNDED),
];

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
        "physics::camera_height_bias",
        call_original,
        &rust,
        &FLOATS,
    )
}
