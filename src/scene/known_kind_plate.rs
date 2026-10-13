//! `scene::known_kind_plate` (0x41F519): a known kind of car's kind of number plate.
//!
//! In:  ebx = which of the known kinds (held to 0 .. 0x21, signed)
//! Out: ebx = the number; everything else preserved
//!
//! `dword_5C5A38` at the place: 0x22 dwords, one for each known kind of car.
//! Two call sites in `scene::load_number_plates`.
//! Manual patch with a private copy: a conditional jump in the first bytes.
//! Self-test: 14 places.

use crate::errors::Refused;
use crate::framework::{EBX, RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("scene::known_kind_plate", rva(0x41F519), thunk, &ORIGINAL)
    .private_copy(0x1C)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const TABLE_RVA: usize = rva(0x5C5A38); // dword_5C5A38

// ---- Rust version ----

pub unsafe fn known_kind_plate(place: u32) -> u32 {
    let place = (place as i32).clamp(0, 0x21) as usize;
    unsafe { game_read(TABLE_RVA + place * 4) }
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EBX, unsafe { known_kind_plate(r.ebx) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for place in [
        0u32,
        1,
        2,
        0x10,
        0x20,
        0x21,
        0x22,
        0x23,
        0x7FFF_FFFF,
        0x8000_0000,
        0xFFFF_FFFF,
        0xFFFF_FFDF,
        0x100,
        5,
    ] {
        check_state(
            &mut t,
            &format!("place {place:#x}"),
            call_original,
            Regs {
                ebx: place,
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            &rust,
        );
    }
    t.finish()
}
