//! `input::yes_no_for_key` (0x407E27): `yes_no_key` for a key the caller already has.
//!
//! In:  eax = the key
//! Out: eax = 1 for yes, 0xFFFFFFFF for no, 2 for the third answer, else 0; everything else
//!      preserved
//! Sets `dword_4F7FD8` to 1 around the call and puts back what it held. One caller,
//! `menu::handle_yes_no`.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::key_to_char::has_layout;
use crate::input::yes_no_key::{answer_for_key, check_keys, yes_no_regions, yes_no_unavailable};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::yes_no_for_key", rva(0x407E27), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn yes_no_for_key(eax: u32) -> Result<u32, crate::errors::BadInput> {
    if !has_layout() {
        return Err(crate::errors::BadInput("no keyboard layout"));
    }
    unsafe { answer_for_key(eax) }
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { yes_no_for_key(r.eax) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &yes_no_regions(), &rust);
}

// ---- self-test: every key ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = yes_no_unavailable() {
        return t.skip(why);
    }
    check_keys(&mut t, call_original, &rust);
    t.finish()
}
