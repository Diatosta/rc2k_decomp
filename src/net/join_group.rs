//! `net::join_group` (0x419354): the session's groups listed (`EnumGroups` with
//! `net::group_found`, which finds the host's "susanivanovafanclub" into `g_netSendTo`), and
//! this machine's player added to that group (`AddPlayerToGroup`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if either failed, else the flags
//!      of `test eax, eax`
//! Callers: `net::join_session`, `net::open_lobby`. DirectPlay's calls from Rust
//! (`net::dplay`), the callback the game's address (its hook). Shadow mode runs the
//! original alone. Self-test: a copy whose DirectPlay is a test object, against the Rust
//! version: joined, either failing.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::create_group::join;
use crate::net::dplay::{ADD_PLAYER_TO_GROUP, ENUM_GROUPS, call};
use crate::net::lobby::{
    DPLAY, DPLAY_METHOD, FLAGS, PLAYER_ID, Part, SEND_TO, Saved, TestObject, com_answer,
    dispatch_lobby, dword, game, p, run_both, test_eax,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::join_group", rva(0x419354), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const GROUP_FOUND: u32 = 0x419391; // net::group_found

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn join_group(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = unsafe { call(DPLAY, ENUM_GROUPS, &[0, game(GROUP_FOUND), 0, 0]) };
    test_eax(&mut r);
    if r.eax != 0 {
        r.set_carry(true);
    } else {
        join(&mut r);
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, join_group);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; push 0; push 0
    let copy =
        match crate::net::lobby::copy(0x419354, 0x3D, &[0x60, 0x6A, 0x00, 0x6A, 0x00], &[], &[]) {
            Ok(c) => c,
            Err(why) => return t.skip(why),
        };
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    let all = [dword(DPLAY), dword(SEND_TO), dword(PLAYER_ID)];
    let saved = Saved::new(&all);
    for (listed, added) in [(0u32, 0u32), (0x8877_0005, 0), (0, 0x8877_0006)] {
        run_both(
            &mut t,
            &format!("EnumGroups {listed:#x}, AddPlayerToGroup {added:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            FLAGS,
            &[],
            &|| {
                p(DPLAY, dp);
                p(SEND_TO, 0x22);
                p(PLAYER_ID, 0x33);
                com_answer(Box::new(move |place, _| {
                    if place == DPLAY_METHOD | ENUM_GROUPS {
                        p(SEND_TO, 0x99);
                        listed
                    } else if place == DPLAY_METHOD | ADD_PLAYER_TO_GROUP {
                        added
                    } else {
                        0
                    }
                }));
            },
            &join_group,
        );
    }
    drop(saved);
    drop(dplay);
    t.finish()
}
