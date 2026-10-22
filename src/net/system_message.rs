//! `net::system_message` (0x41976F): a DirectPlay system message (sender 0) handled, by its
//! first dword: 3 (DPSYS_CREATEPLAYERORGROUP) the sync table to be cleared again
//! (`dword_52AD8C` 1); 5 (DPSYS_DESTROYPLAYERORGROUP) the same and, for a player (+4 is 1),
//! that player (+8) taken out of the group (`DeletePlayerFromGroup`, `IDirectPlay4A`
//! +0x1C, `g_netSendTo`); 0x31 (DPSYS_SESSIONLOST) `net::destroy_connection`; 0x101
//! (DPSYS_HOST) this machine the host (`g_netOwnRecord` bit 0x10 set, bit 1 ready cleared),
//! `dword_52AD8C` 1 and the stage item open (`dword_52AC2C` 1). Others: nothing.
//!
//! In:  ebx = the message
//! Out: every register kept (`pusha` .. `popa`); the flags of its last compare or `and`,
//!      `destroy_connection`'s for 0x31, after `DeletePlayerFromGroup` the method's (not
//!      modelled: the Rust version leaves the compare's); the one caller does not read them
//! Caller: `net::take_messages`. `destroy_connection` through `part`,
//! `DeletePlayerFromGroup` from Rust (`net::dplay`, section 15). Refuses nothing. Shadow
//! mode runs the original alone. Self-test: a copy whose variables are the test's, whose
//! call goes to a noting stand-in and whose DirectPlay is a test object, against the Rust
//! version: each kind.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{DELETE_PLAYER_FROM_GROUP, method};
use crate::net::lobby::{FLAGS, TestObject, answer, cmp_flags, run_both, set_sign};
use crate::net::message_vars::{MessageVars, TestVars, as_rust, get, get8, set, set8};
use crate::net::{DESTROY, Part, dispatch_net, net_entry};
use rc2k_core::net::players::{A_PLAYER, System, made_host};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::system_message", rva(0x41976F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const CREATE_PLAYER_OR_GROUP: u32 = 3;
pub const DESTROY_PLAYER_OR_GROUP: u32 = 5;
pub const SESSION_LOST: u32 = 0x31;
pub const HOST: u32 = 0x101;

/// The Rust version on the variables at `v`.
pub fn system_message_with(r: &mut Regs, v: &MessageVars, part: Part) -> Result<(), Refused> {
    let kind = get(r.ebx);
    // Converted: `rc2k_core::net::players` (`System`, `made_host`).
    match System::of(kind) {
        System::Created => {
            cmp_flags(r, kind, CREATE_PLAYER_OR_GROUP);
            set(v.sync_first, 1);
        }
        System::Destroyed => {
            set(v.sync_first, 1);
            let what = get(r.ebx + 4);
            cmp_flags(r, what, A_PLAYER);
            if what == A_PLAYER {
                unsafe {
                    method(
                        get(v.dplay),
                        DELETE_PLAYER_FROM_GROUP,
                        &[get(v.send_to), get(r.ebx + 8)],
                    )
                };
            }
        }
        System::SessionLost => {
            let mut c = Regs { eax: kind, ..*r };
            part(DESTROY, &mut c)?;
            r.eflags = c.eflags;
        }
        System::Host => {
            let own = made_host(get8(v.own));
            set8(v.own, own);
            r.set_carry(false);
            r.set_zero(own == 0);
            set_sign(r, own as i8 as i32);
            set(v.sync_first, 1);
            set(v.stage_item, 1);
        }
        System::Other => cmp_flags(r, kind, HOST),
    }
    Ok(())
}

/// The Rust version.
pub fn system_message(r: &mut Regs, part: Part) -> Result<(), Refused> {
    system_message_with(r, &MessageVars::game(), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, system_message);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, system_message)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let tv = TestVars::new(0);
    // pusha; mov eax, [ebx]; cmp eax, 3
    let entry = [0x60, 0x8B, 0x03, 0x83, 0xF8, 0x03];
    let copy = match tv.copy(0x41976F, 0x76, &entry, &[0x44], &[DESTROY], 8) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let v = tv.v;
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    let mut message = [0u32; 3];
    let m = message.as_mut_ptr() as u32;
    // (the kind, +4, the own record's byte)
    for (kind, what, own) in [
        (CREATE_PLAYER_OR_GROUP, 1u32, 0x01u8),
        (DESTROY_PLAYER_OR_GROUP, A_PLAYER, 0x01),
        (DESTROY_PLAYER_OR_GROUP, 2, 0x01),
        (SESSION_LOST, 0, 0x01),
        (HOST, 0, 0x01),
        (HOST, 0, 0x81),
        (HOST, 0, 0xFF),
        (0x22, 1, 0x01),
        (0, 1, 0x01),
        (0x8000_0101, 1, 0x01),
    ] {
        let called = kind == DESTROY_PLAYER_OR_GROUP && what == A_PLAYER;
        run_both(
            &mut t,
            &format!("kind {kind:#x}, +4 {what}, own {own:#x}"),
            copy.addr(),
            Regs {
                ebx: m,
                ..Regs::SENTINEL
            },
            &[tv.region(), (m, 12)],
            if called { 0 } else { FLAGS },
            &[],
            &|| {
                set(m, kind);
                set(m + 4, what);
                set(m + 8, 0x0000_7005);
                set(v.dplay, dp);
                set(v.send_to, 0x0000_7100);
                set8(v.own, own);
                set(v.sync_first, 0);
                set(v.stage_item, 0);
                // destroy_connection's flags handed on.
                answer(Box::new(|_, r| r.set_zero(true)));
            },
            &as_rust(|r, part| system_message_with(r, &v, part)),
        );
    }
    let _ = &mut message;
    drop(dplay);
    t.finish()
}
