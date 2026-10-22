//! `net::receive_message` (0x419722): one message taken from DirectPlay: its size
//! (`dword_5AB280`) set to 0x100, `Receive` (`IDirectPlay4A` +0x64: the sender into
//! `dword_5AB278`, the addressee into `unk_5AB27C`, flags 1 DPRECEIVE_ALL, the bytes into
//! `unk_52A968`) through `g_dplay`; the size made 0 if it failed (no message, or one too
//! big for the buffer).
//!
//! In:  nothing
//! Out: eax = the sender (0: a system message), ebx = the bytes (`unk_52A968`), ecx = the
//!      size (0: none); edx kept (pushed and popped), esi, edi, ebp kept (the method's);
//!      the flags of `test eax, eax` on the method's result
//! Caller: `net::take_messages`. `Receive` from Rust (`net::dplay`, section 15). Refuses
//! nothing (no check of `g_dplay`: its caller makes it). Shadow mode runs the original
//! alone (a message is taken). Self-test: a copy whose variables are the test's and whose
//! DirectPlay is a test object writing a message, against the Rust version: taken, none.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{RECEIVE, method};
use crate::net::lobby::{DPLAY_METHOD, FLAGS, TestObject, com_answer, run_both, test_eax};
use crate::net::message_vars::{MessageVars, TestVars, as_rust, get, set};
use crate::net::{Part, dispatch_net, net_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::receive_message", rva(0x419722), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The buffer's size.
pub const ROOM: u32 = 0x100;
/// DPRECEIVE_ALL.
const RECEIVE_ALL: u32 = 1;

/// The Rust version on the variables at `v`.
pub fn receive_message_with(r: &mut Regs, v: &MessageVars) {
    set(v.size, ROOM);
    let result = unsafe {
        method(
            get(v.dplay),
            RECEIVE,
            &[v.from, v.to, RECEIVE_ALL, v.data, v.size],
        )
    };
    if result != 0 {
        set(v.size, 0);
    }
    r.eax = result;
    test_eax(r);
    r.eax = get(v.from);
    r.ebx = v.data;
    r.ecx = get(v.size);
}

/// The Rust version.
pub fn receive_message(r: &mut Regs, _part: Part) -> Result<(), Refused> {
    receive_message_with(r, &MessageVars::game());
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, receive_message);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, receive_message)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let tv = TestVars::new(0);
    // push edx; mov dword_5AB280, 100h
    let entry = [
        0x52, 0xC7, 0x05, 0x80, 0xB2, 0x5A, 0x00, 0x00, 0x01, 0x00, 0x00,
    ];
    let copy = match tv.copy(0x419722, 0x4D, &entry, &[], &[], 10) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let v = tv.v;
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    // (the result, the sender, the size written)
    for (result, sender, size) in [
        (0u32, 0x0000_7002u32, 5u32),
        (0, 0, 0x18),
        (0, 0x0000_7003, 0x100),
        (0x8877_03E8, 0, 0),
        (0x8877_001E, 0x0000_7004, 0x180),
    ] {
        run_both(
            &mut t,
            &format!("Receive gives {result:#x}, from {sender:#x}, {size:#x} bytes"),
            copy.addr(),
            Regs::SENTINEL,
            &[tv.region()],
            FLAGS,
            &[],
            &|| {
                set(v.dplay, dp);
                set(v.from, 0xAAAA_AAAA);
                set(v.size, 0xAAAA_AAAA);
                com_answer(Box::new(move |place, args| {
                    if place != DPLAY_METHOD | RECEIVE {
                        return 0;
                    }
                    // this, from, to, flags, data, size
                    set(args[1], sender);
                    set(args[2], 0x0000_7001);
                    for k in 0..size.min(ROOM) {
                        unsafe { ((args[4] + k) as *mut u8).write(k as u8 ^ 0x5A) };
                    }
                    set(args[5], size);
                    result
                }));
            },
            &as_rust(|r, _| {
                receive_message_with(r, &v);
                Ok(())
            }),
        );
    }
    drop(dplay);
    t.finish()
}
