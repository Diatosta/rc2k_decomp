//! `net::create_dplay` (0x41861D): "NET: Create dplay", and DirectPlay's object made
//! (`CoCreateInstance` of CLSID_DirectPlay `stru_5AB330`, IID_IDirectPlay4A `stru_5AB310`,
//! in-process) into `g_dplay`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if it failed, else the flags of
//!      `test eax, eax` on its result
//! Caller: `net::find_connections`. Its calls through `part` (the log; `CoCreateInstance`
//! from Rust, `net::dplay::create_instance`: the platform). Shadow mode runs the original
//! alone (it makes an object). Self-test: a copy whose calls go to stand-ins, against the
//! Rust version: made, failing.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    CO_CREATE, DPLAY, FLAGS, LOG_L2_NL, Part, a, com_answer, dispatch_lobby, log_text, run_both,
    test_eax,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::create_dplay", rva(0x41861D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXT: usize = 0x5C2E3A; // "NET: Create dplay"
pub const CLSID_DIRECTPLAY: usize = 0x5AB330; // stru_5AB330
pub const IID_DIRECTPLAY4A: usize = 0x5AB310; // stru_5AB310

/// `CoCreateInstance` through `part` (eax the class, ebx the interface, ecx where): its
/// result in eax, the other registers as they were.
pub fn create(
    r: &mut Regs,
    class: u32,
    interface: u32,
    out: u32,
    part: Part,
) -> Result<(), Refused> {
    let mut c = Regs {
        eax: class,
        ebx: interface,
        ecx: out,
        ..*r
    };
    part(CO_CREATE, &mut c)?;
    r.eax = c.eax;
    Ok(())
}

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn create_dplay(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    log_text(LOG_L2_NL, a(TEXT), &r, part)?;
    create(
        &mut r,
        a(CLSID_DIRECTPLAY),
        a(IID_DIRECTPLAY4A),
        a(DPLAY),
        part,
    )?;
    test_eax(&mut r);
    if r.eax != 0 {
        r.set_carry(true);
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, create_dplay);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; pusha; lea eax, aNetCreateDplay
    let copy = match crate::net::lobby::copy(
        0x41861D,
        0x31,
        &[0x60, 0x60, 0x8D, 0x05, 0x3A, 0x2E, 0x5C, 0x00],
        &[0x8, 0x21],
        &[LOG_L2_NL],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    for result in [0u32, 0x8004_0154] {
        run_both(
            &mut t,
            &format!("CoCreateInstance gives {result:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &[],
            FLAGS,
            &[],
            &|| com_answer(Box::new(move |_, _| result)),
            &create_dplay,
        );
    }
    t.finish()
}
