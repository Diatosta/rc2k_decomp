//! `net::create_group` (0x4192EA): the host's group made, which every player joins and every
//! message goes to: a DPNAME (`dword_5AB29C`) named "susanivanovafanclub" (`0x5AB25C`),
//! `CreateGroup` into `g_netSendTo` (no data, flags 0), and this machine's player added to
//! it (`AddPlayerToGroup`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if either failed, else the flags
//!      of `test eax, eax`
//! Callers: `net::host_session`, `net::open_lobby`. Its call through `part`; DirectPlay's
//! from Rust (`net::dplay`). Shadow mode runs the original alone. Self-test: a copy whose
//! call goes to a stand-in and whose DirectPlay is a test object, against the Rust version:
//! made, either failing.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::create_player::dp_name;
use crate::net::dplay::{ADD_PLAYER_TO_GROUP, CREATE_GROUP, call};
use crate::net::lobby::{
    DP_NAME, DPLAY, DPLAY_METHOD, FLAGS, PLAYER_ID, Part, SEND_TO, Saved, TestObject, ZERO_MEMORY,
    a, com_answer, dispatch_lobby, dword, fill, g, p, region, run_both, test_eax,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::create_group", rva(0x4192EA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const GROUP_NAME: usize = 0x5AB25C; // "susanivanovafanclub"

/// `AddPlayerToGroup(g_netSendTo, g_netPlayerId)`, then the flags; shared with
/// `net::join_group`.
pub fn join(r: &mut Regs) {
    r.eax = unsafe { call(DPLAY, ADD_PLAYER_TO_GROUP, &[g(SEND_TO), g(PLAYER_ID)]) };
    test_eax(r);
    if r.eax != 0 {
        r.set_carry(true);
    }
}

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn create_group(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    dp_name(&r, a(GROUP_NAME), part)?;
    r.eax = unsafe { call(DPLAY, CREATE_GROUP, &[a(SEND_TO), a(DP_NAME), 0, 0, 0]) };
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
    dispatch_lobby(&HOOK, r, call_original, create_group);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; push ebx; push ecx; lea ebx, dword_5AB29C
    let copy = match crate::net::lobby::copy(
        0x4192EA,
        0x6A,
        &[0x60, 0x53, 0x51, 0x8D, 0x1D, 0x9C, 0xB2, 0x5A, 0x00],
        &[0xE],
        &[ZERO_MEMORY],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    let all = [
        dword(DPLAY),
        region(DP_NAME, 0x10),
        dword(SEND_TO),
        dword(PLAYER_ID),
    ];
    let saved = Saved::new(&all);
    for (made, added) in [(0u32, 0u32), (0x8877_0005, 0), (0, 0x8877_0006)] {
        run_both(
            &mut t,
            &format!("CreateGroup {made:#x}, AddPlayerToGroup {added:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            FLAGS,
            &[],
            &|| {
                p(DPLAY, dp);
                fill(DP_NAME, 0x10, 92);
                p(SEND_TO, 0x22);
                p(PLAYER_ID, 0x33);
                com_answer(Box::new(move |place, args| {
                    if place == DPLAY_METHOD | CREATE_GROUP {
                        unsafe { (args[1] as *mut u32).write_unaligned(0x4444) };
                        made
                    } else if place == DPLAY_METHOD | ADD_PLAYER_TO_GROUP {
                        added
                    } else {
                        0
                    }
                }));
            },
            &create_group,
        );
    }
    drop(saved);
    drop(dplay);
    t.finish()
}
