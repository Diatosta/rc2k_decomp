//! `scene::turn_stage_object_point` (0x426A5A): for the loading kinds 0x190 .. 0x257
//! (`g_loadingKind`), a stage object's point turned a quarter and shifted: eax = ecx,
//! ecx = -eax, ebx less `dword_613FC4`.
//!
//! In:  eax, ecx, ebx the point
//! Out: those three as said (else as they came); everything else preserved
//! One caller, 0x42686F (`scene::load_stage_objects`'s point loop). Shadow mode: both,
//! compared.
//! Self-test: the original against the Rust version for loading kinds around the bounds
//! (the game's variable, put back).

use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, ECX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva,
};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::turn_stage_object_point",
    rva(0x426A5A),
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

/// g_loadingKind, dword_613FC4.
const KIND_RVA: usize = rva(0x634FE8);
const SHIFT_RVA: usize = rva(0x613FC4);

// ---- Rust version ----

/// (eax, ecx, ebx), or None when the kind leaves them.
pub fn turn_stage_object_point(
    kind: u32,
    shift: u32,
    p: (u32, u32, u32),
) -> Option<(u32, u32, u32)> {
    (0x190..0x258)
        .contains(&kind)
        .then(|| (p.1, p.0.wrapping_neg(), p.2.wrapping_sub(shift)))
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (kind, shift) = unsafe { (game_read(KIND_RVA), game_read(SHIFT_RVA)) };
    Ok(
        match turn_stage_object_point(kind, shift, (r.eax, r.ecx, r.ebx)) {
            Some((a, c, b)) => Left::nothing().with(EAX, a).with(ECX, c).with(EBX, b),
            None => Left::nothing(),
        },
    )
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = [at(KIND_RVA), at(SHIFT_RVA)];
    let saved = capture(&all);
    for kind in [0u32, 0x18F, 0x190, 0x200, 0x257, 0x258, 0xFFFF_FFFF] {
        let fill = || unsafe {
            game_write(KIND_RVA, kind);
            game_write(SHIFT_RVA, 0x1234u32);
        };
        check_state(
            &mut t,
            &format!("kind {kind:#x}"),
            call_original,
            Regs {
                eax: 0x100,
                ecx: 0xFFFF_FF00,
                ebx: 0x10,
                ..Regs::SENTINEL
            },
            &fill,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
