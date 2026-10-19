//! `net::dplay_send` (0x4196FF): DirectPlay's `Send` (`IDirectPlay4A` +0x68) from this
//! machine's player (`g_netPlayerId`) through `g_dplay`.
//!
//! In:  eax = to whom (a player or the group), ebx = the bytes, ecx = how many, edx = the
//!      flags (DPSEND_*)
//! Out: every register kept (eax, ecx and edx pushed and popped); success: the flags of
//!      `test eax, eax` on the result 0 (zero set, carry clear); failure: those of the
//!      result, then carry set
//! Six callers (`net::send_message`, `net::send_chat`, `net::race_wait`, `net::lobby_loop`,
//! `net::take_messages` twice). `Send` from Rust (`net::dplay`, section 15). Refuses nothing
//! (no check of `g_dplay`, as the original). Shadow mode runs the original alone (the
//! message is sent). Self-test: a copy whose two variables are the test's and whose
//! DirectPlay is a test object noting its arguments, against the Rust version: sent, failed.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{SEND, method};
use crate::net::lobby::{DPLAY_METHOD, FLAGS, TestObject, com_answer, run_both, test_eax};
use crate::net::message_vars::{MessageVars, TestVars, as_rust, get, set};
use crate::net::{Part, dispatch_net};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::dplay_send", rva(0x4196FF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version on the variables at `v`.
pub fn dplay_send_with(r: &mut Regs, v: &MessageVars) {
    let result = unsafe {
        method(
            get(v.dplay),
            SEND,
            &[get(v.player_id), r.eax, r.edx, r.ebx, r.ecx],
        )
    };
    let mut f = Regs { eax: result, ..*r };
    test_eax(&mut f);
    if result != 0 {
        f.set_carry(true);
    }
    r.eflags = f.eflags;
}

/// The Rust version.
pub fn dplay_send(r: &mut Regs, _part: Part) -> Result<(), Refused> {
    dplay_send_with(r, &MessageVars::game());
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, dplay_send);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let tv = TestVars::new(0);
    // push eax; push ecx; push edx; push ecx; push ebx
    let copy = match tv.copy(0x4196FF, 0x23, &[0x50, 0x51, 0x52, 0x51, 0x53], &[], &[], 2) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let v = tv.v;
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    for result in [0u32, 0x8877_0010, 0x0000_0001] {
        run_both(
            &mut t,
            &format!("Send gives {result:#x}"),
            copy.addr(),
            Regs {
                eax: 0x0000_7001,
                ebx: 0x00AB_C000,
                ecx: 0x22,
                edx: 1,
                ..Regs::SENTINEL
            },
            &[tv.region()],
            FLAGS,
            &[],
            &|| {
                set(v.dplay, dp);
                set(v.player_id, 0x0000_6001);
                com_answer(Box::new(move |place, _| {
                    if place == DPLAY_METHOD | SEND {
                        result
                    } else {
                        0
                    }
                }));
            },
            &as_rust(|r, _| {
                dplay_send_with(r, &v);
                Ok(())
            }),
        );
    }
    drop(dplay);
    t.finish()
}
