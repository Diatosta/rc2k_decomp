//! `race::single_rally_race` (0x4843EC): a single rally's (and the other menus') race: the
//! track looked up, the settings and the players' records copied into the race's
//! variables, the race run (`race::race_from_menu`, the sound objects and the screen modes
//! round it), the results copied back. See `race/single_rally.rs`.
//!
//! In:  eax = 1 when the loading picture is not to be shown after, ebx = the kind (3:
//!      the rally's times copied first)
//! Out: every register preserved (`pusha` / `popa`)
//!
//! About 50 callers (the menus' races; each `cmp`s after, reading no flags). A `jnz` in
//! its first bytes: private copy of its body (0x1184 bytes, its 17 calls and 5 jumps to
//! the fatal stubs relocated), manual patch. Raises fatal 0x80739281 "Unknown Shaun
//! TrakNum" (a track not in `g_trakPakHeader`) from Rust; refuses a player's record
//! missing (first). Shadow mode runs the original alone (it runs the race). Self-test: a
//! copy whose calls go to stand-ins, against the Rust version on the same, on the game's
//! players and tracks (skipped without them), the variables and records put back.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::single_rally::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALL_OFFSETS: [usize; 17] = [
    0x6, 0x4A, 0x4F, 0x54, 0xA2D, 0xA32, 0xA37, 0xA3C, 0xA41, 0xA50, 0xA55, 0xA5A, 0x115D, 0x1162,
    0x1167, 0x116C, 0x117D,
];
const JUMPS_OUT: [usize; 5] = [0x82, 0xD0F, 0xD32, 0x1071, 0x1094];

pub static HOOK: RawHook = RawHook::new("race::single_rally_race", rva(0x4843EC), thunk, &ORIGINAL)
    .private_copy(0x1184)
    .relocate_calls(&[
        0x6, 0x4A, 0x4F, 0x54, 0xA2D, 0xA32, 0xA37, 0xA3C, 0xA41, 0xA50, 0xA55, 0xA5A, 0x115D,
        0x1162, 0x1167, 0x116C, 0x117D, 0x82, 0xD0F, 0xD32, 0x1071, 0x1094,
    ])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    // It runs a whole race: a gated thunk would count the game's thread as inside a
    // hook throughout, and every test's pause wait out its 2 s (2026-10-18).
    ungated
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { single_rally_race(r, &mut game_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::audio::test_copy::{Copy, make};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let second = unsafe { (game_addr(rva(0x75598C)) as *const u32).read_unaligned() };
    let (p1, p2) = unsafe { (player(1), player(second)) };
    if p1 == 0 || p2 == 0 {
        return t.skip("no player records");
    }
    // The first track's number, so the look-up finds one.
    let first_track = unsafe { ((game_addr(rva(0x734BC6)) + 0x68) as *const u32).read_unaligned() };
    if first_track == u32::MAX {
        return t.skip("no tracks");
    }
    let copy = match make(
        &HOOK,
        &Copy {
            address: 0x4843EC,
            len: 0x1184,
            entry: &[0x60, 0x83, 0xFB, 0x03, 0x75, 0x05],
            calls: &CALL_OFFSETS,
            jumps_out: &JUMPS_OUT,
        },
        &|c| {
            CALLS
                .iter()
                .position(|&k| rva(k) == c)
                .map(|n| crate::install::stand_ins::stand_in(n as u32 + 1))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    crate::install::stand_ins::set_handler(crate::race::level_init::note);
    let mut regions: Vec<Region> = written();
    regions.extend([(p1, PLAYER_LEN), (p2, PLAYER_LEN)]);
    regions.push((game_addr(rva(0x75ACD4)) as u32, 4));
    regions.push((game_addr(rva(0x755984)) as u32, 12));
    let saved = capture(&regions);
    for (kind, start, mode, other) in [
        (1u32, 0u32, 0u32, 0u32),
        (3, 1, 0, 1),
        (1, 0, 5, 0),
        (2, 1, 4, 1),
        (1, 0, 10, 0),
    ] {
        crate::race::race_cars::run_cars(
            &mut t,
            &format!("kind {kind}, start {start}, mode {mode}, other {other}"),
            &copy,
            &regions,
            Regs {
                eax: start,
                ebx: kind,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&regions, &saved);
                game_write(rva(0x75ACD4), first_track);
                game_write(rva(0x755988), mode);
                game_write(rva(0x755984), other);
            },
            &|r| unsafe { single_rally_race(r, &mut crate::race::mud::test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
