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
//! Self-test: `physics/camera_view.rs` (`view_test`), modes and times by lot, three
//! precisions.

use crate::errors::Refused;
use crate::framework::{EAX, ESI, RawHook, Regs, hook_asm, rva};
use crate::physics::camera_view::{rd, record, share_done, view_regions, view_test};
use crate::physics::flow::{base, x87_run};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
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

/// The Rust version.
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

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { ease_view_blend(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &view_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    view_test(&HOOK, call_original, &rust)
}
