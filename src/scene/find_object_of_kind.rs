//! `scene::find_object_of_kind` (0x422298): the first object (1 .. 0xFF) of a kind.
//!
//! In:  eax = the kind
//! Out: eax = the object, 0 if none; the rest preserved
//!
//! `g_objectKinds` (0x5CD27C, 0x100 dwords; object 0 is never looked at). 16 call sites
//! (loadlevel_init, the carousels and car pictures, the replay cameras). Self-test: the
//! original and the Rust version on the game's table, kinds of the test's, put back
//! afterwards.

use crate::errors::Refused;
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use rc2k_core::scene::kinds::OBJECTS;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::find_object_of_kind",
    rva(0x422298),
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

const KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds

/// `scene::find_object_of_kind` (see the header).
pub unsafe fn find_object_of_kind(kind: u32) -> u32 {
    // Converted: `rc2k_core::scene::kinds::first_of_kind`.
    rc2k_core::scene::kinds::first_of_kind(|n| unsafe { game_read(KINDS_RVA + n * 4) }, kind)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { find_object_of_kind(r.eax) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let table: Region = (game_addr(KINDS_RVA) as u32, 4 * OBJECTS as usize);
    let saved = capture(&[table]);
    let fill = || unsafe {
        for n in 0..OBJECTS {
            game_write(KINDS_RVA + 4 * n as usize, (n * 7) % 0x60);
        }
    };
    // Kind 0 is at object 0 (never looked at) and next at 0x60; 0x77 is nowhere.
    for kind in [0u32, 7, 0x5F, 0x77, 0x31] {
        check_state(
            &mut t,
            &format!("kind {kind:#x}"),
            call_original,
            Regs {
                eax: kind,
                ..Regs::SENTINEL
            },
            &fill,
            &[table],
            &rust,
        );
    }
    restore(&[table], &saved);
    t.finish()
}
