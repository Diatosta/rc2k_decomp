//! `menu::replay_item_c` (0x470529): `race::replay_mode_2` and back (a 6-byte stub: `call;
//! retn`), the third replay item of the review menus (see `menu/replay_items.rs`; what it
//! shows is a reading of the viewer's mode).
//!
//! In:  nothing
//! Out: the registers as `replay_mode_2` leaves them
//!
//! An item action in the review menus' tables (0x74E5CC ..). Ungated thunk; shadow mode
//! runs the original alone. Manual patch with a private copy (the whole stub, its call
//! relocated). Self-test: a copy whose call goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::parts::{Part, copy, dispatch_parted, run_both, test_part};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::replay_item_c", rva(0x470529), thunk, &ORIGINAL)
    .private_copy(6)
    .relocate_calls(&[0])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const CALLEES: [usize; 1] = [0x41CD9D]; // race::replay_mode_2

/// The Rust version: the registers it leaves.
pub unsafe fn replay_item_c(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(0, &mut r)?;
    Ok(r)
}

fn game_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    *r = crate::race::replay_mode_2::replay_mode_2(r, &mut crate::race::replay_mode::game_part)?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parted(
        &HOOK,
        r,
        call_original,
        &|r, part| unsafe { replay_item_c(r, part) },
        game_part,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x470529,
        6,
        &[0xE8, 0x6F, 0xC8, 0xFA, 0xFF],
        &[0],
        &CALLEES,
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    run_both(
        &mut t,
        "once",
        &copy,
        &[],
        &|| {},
        Regs::SENTINEL,
        false,
        &|r| unsafe { replay_item_c(r, &mut test_part) },
    );
    t.finish()
}
