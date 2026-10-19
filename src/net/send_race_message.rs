//! `net::send_race_message` (0x46449E): `call net::send_message; retn`.
//!
//! In:  ebx = the bytes, ecx = how many
//! Out: as `net::send_message`
//!
//! See `net/mod.rs`. Three callers: `net::send_inputs`, `net::send_position`,
//! `net::send_extra`. Its call through `part`. A call in its first five bytes: private copy
//! (6 bytes, the call relocated), manual patch. Shadow mode runs the original alone.
//! Self-test: a copy whose call goes to a noting stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::{Part, SEND_MESSAGE, dispatch_net};
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("net::send_race_message", rva(0x46449E), thunk, &ORIGINAL)
    .private_copy(6)
    .relocate_calls(&[0])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// call sub_41736D; retn
const ENTRY: [u8; 6] = [0xE8, 0xCA, 0x2E, 0xFB, 0xFF, 0xC3];

/// The Rust version.
pub fn send_race_message(r: &mut Regs, part: Part) -> Result<(), Refused> {
    part(SEND_MESSAGE, r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, send_race_message);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_with(0x46449E, 6, &ENTRY, &[0], &[SEND_MESSAGE], &[], &[], 0) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    compare(
        &mut t,
        "the call",
        &copy,
        Regs::SENTINEL,
        &|| {},
        &[],
        &|r, part| send_race_message(r, part),
        false,
    );
    t.finish()
}
