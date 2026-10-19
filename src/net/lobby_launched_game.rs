//! `net::lobby_launched_game` (0x418DF4): the game as a lobby launched it (`g_lobbyLaunched`
//! 1, instead of the front end): `sub_4154CD` (the front end's set-up, agent F's), the
//! network game (`net::start_link`, mode 3), then the 3D tidied (`race::render3d_tidy`,
//! `race::objects_tidy`, `nullsub_6`, `race::release_textures`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `system::main_loop` (whose Rust version calls this one). Its calls through `part`
//! (`sub_4154CD` through the game). The whole game runs inside it: ungated. Shadow mode runs
//! the original alone. Self-test: a copy whose calls go to stand-ins, against the Rust
//! version, launched or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    LOBBY_LAUNCHED, NULLSUB_6, OBJECTS_TIDY, Part, RELEASE_TEXTURES, RENDER3D_TIDY,
    START_FRONT_END, Saved, dispatch_lobby, dword, g, p, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::lobby_launched_game", rva(0x418DF4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const START_LINK: u32 = 0x416F50; // net::start_link

/// The Rust version: the registers it leaves (the entry's).
pub fn lobby_launched_game(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if g(LOBBY_LAUNCHED) == 1 {
        part(START_FRONT_END, &mut r)?;
        r.eax = 3;
        for place in [
            START_LINK,
            RENDER3D_TIDY,
            OBJECTS_TIDY,
            NULLSUB_6,
            RELEASE_TEXTURES,
        ] {
            part(place, &mut r)?;
        }
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, lobby_launched_game);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; cmp g_lobbyLaunched, 1
    let copy = match crate::net::lobby::copy(
        0x418DF4,
        0x2F,
        &[0x60, 0x83, 0x3D, 0x90, 0xB2, 0x5A, 0x00, 0x01],
        &[0xA, 0x14, 0x19, 0x1E, 0x23, 0x28],
        &[
            START_FRONT_END,
            START_LINK,
            RENDER3D_TIDY,
            OBJECTS_TIDY,
            NULLSUB_6,
            RELEASE_TEXTURES,
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [dword(LOBBY_LAUNCHED)];
    let saved = Saved::new(&all);
    for launched in [0u32, 1, 2] {
        run_both(
            &mut t,
            &format!("launched {launched}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            0,
            &[],
            &|| p(LOBBY_LAUNCHED, launched),
            &lobby_launched_game,
        );
    }
    drop(saved);
    t.finish()
}
