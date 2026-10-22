//! `physics::ease_view_blend` (0x4385EB): with the mode's record's blend (+0x58) and
//! change time (+0x3C) not 0: the share done (`physics/camera_view.rs`; 1 less it for a
//! blend below 0) between `flt_608B20` and `flt_608B24` into `unk_71A6A0`, and between
//! `flt_608B28` and `flt_608B2C`, x `flt_5C5D98`, negated, `fistp`ed into
//! `dword_608B38`.
//!
//! In:  nothing
//! Out: esi the mode's record; eax the time held when it ran; the rest kept; no flags
//!      read
//!
//! Caller: `sub_437A53`.
//! Self-test: `physics/camera_view.rs` (`view_test_floats`), modes and times by lot,
//! exact at double precision.

use crate::errors::Refused;
use crate::framework::{EAX, ESI, RawHook, Regs, hook_asm, rva};
use crate::physics::camera_view::{
    rd, record, record_plain, share_done, share_done_plain, view_regions, view_test_floats,
};
use crate::physics::flow::{a, base, g, p, x87_run};
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, ROUNDED, Tolerance, dispatch_state_floats, floats_entry,
};
use rc2k_core::physics::camera_view::ease_blend;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::ease_view_blend", rva(0x4385EB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (the exact one, for `physics::update_camera`).
pub unsafe fn ease_view_blend(_: &Regs) -> Left {
    let esi = record();
    let left = Left::nothing().with(ESI, esi);
    if rd(esi + 0x58) == 0 || rd(esi + 0x3C) == 0 {
        return left;
    }
    let eax = share_done(esi);
    if (rd(esi + 0x58) as i32) < 0 {
        x87_run!(
            ".byte 0xD9, 0xE8", // fld1
            "fsub dword ptr [{b} + 0x205260]",
            "fstp dword ptr [{b} + 0x205260]";
            b = base()
        );
    }
    x87_run!(
        "fld dword ptr [{b} + 0x208B24]",
        "fsub dword ptr [{b} + 0x208B20]",
        "fmul dword ptr [{b} + 0x205260]",
        "fadd dword ptr [{b} + 0x208B20]",
        "fstp dword ptr [{b} + 0x31A6A0]",
        "fld dword ptr [{b} + 0x208B2C]",
        "fsub dword ptr [{b} + 0x208B28]",
        "fmul dword ptr [{b} + 0x205260]",
        "fadd dword ptr [{b} + 0x208B28]",
        "fmul dword ptr [{b} + 0x1C5D98]",
        ".byte 0xD9, 0xE0", // fchs
        "fistp dword ptr [{b} + 0x208B38]";
        b = base()
    );
    left.with(EAX, eax)
}

/// The converted version (`rc2k_core::physics::camera_view`, plain floats): the hook's.
// Converted: `rc2k_core::physics::camera_view::{share_done, ease_blend}`.
pub unsafe fn ease_view_blend_plain(_: &Regs) -> Left {
    let esi = record_plain();
    let left = Left::nothing().with(ESI, esi);
    if rd(esi + 0x58) == 0 || rd(esi + 0x3C) == 0 {
        return left;
    }
    let eax = share_done_plain(esi);
    let f = |x: usize| f32::from_bits(g(x));
    let b = ease_blend(
        f(0x605260),
        (rd(esi + 0x58) as i32) < 0,
        [f(0x608B20), f(0x608B24), f(0x608B28), f(0x608B2C)],
        f(0x5C5D98),
    );
    p(0x605260, b.share.to_bits());
    p(0x71A6A0, b.between.to_bits());
    p(0x608B38, b.tilt as u32);
    left.with(EAX, eax)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { ease_view_blend_plain(r) })
}

/// The float places and what they are held to at extended precision.
const FLOATS: [(usize, usize, Tolerance); 3] = [
    (0x605260, 4, FLOAT),
    (0x71A6A0, 4, FLOAT),
    (0x608B38, 4, ROUNDED),
];

extern "C" fn dispatch(r: &mut Regs) {
    let memory = FLOATS.map(|(x, len, tl)| ((a(x), len), tl));
    let floats = Floats {
        memory: &memory,
        regs: &[],
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
        "physics::ease_view_blend",
        call_original,
        &rust,
        &FLOATS,
    )
}
