//! `net::setting_label` (0x416AF8): a setting's label as a button (`net::draw_button`, flags
//! 0, x 0x190, y ebp), and ebp 0x20 lower. (The code after its `ret`, 0x416B0D .. 0x416B52,
//! is reached by nothing.)
//!
//! In:  eax = the string, esi = the item, ebp = y
//! Out: ebx = 0, ecx = 0x190, edx = the y, ebp + 0x20; the rest as the call leaves it (it
//!      keeps every register); no `pusha`
//! Caller: `net::settings_frame` (four times). Its call through `part`. Shadow mode runs the
//! original alone (it draws). Self-test: a copy whose call goes to a stand-in, against the
//! Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{Part, dispatch_lobby, run_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::setting_label", rva(0x416AF8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DRAW_BUTTON: u32 = 0x416CE0; // net::draw_button

/// The Rust version: the registers it leaves.
pub fn setting_label(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.ebx = 0;
    r.ecx = 0x190;
    r.edx = r.ebp;
    part(DRAW_BUTTON, &mut r)?;
    r.ebp = r.ebp.wrapping_add(0x20);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, setting_label);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov ebx, 0
    let copy = match crate::net::lobby::copy(
        0x416AF8,
        0x15,
        &[0xBB, 0x00, 0x00, 0x00, 0x00],
        &[0xC],
        &[DRAW_BUTTON],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    run_both(
        &mut t,
        "a label",
        copy.addr(),
        Regs {
            eax: 0x163,
            esi: 1,
            ebp: 0x92,
            ..Regs::SENTINEL
        },
        &[],
        0,
        &[],
        &|| {},
        &setting_label,
    );
    t.finish()
}
