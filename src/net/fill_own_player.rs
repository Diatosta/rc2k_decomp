//! `net::fill_own_player` (0x4181A3): this machine's player set up for the network game from
//! player 1 (`g_player1`, 0x755A3C on): its name (`strings::copy_printable`, 0x1F room) to
//! `byte_52AB70`; its record (`byte_52AD64`): state 0, the car, its colours (+0x30), the
//! nationality, the stage (`dword_75ACD4`; 0 is made 0x29 with direction 0) and its
//! direction (`dword_75ACF0`), the race number, a random byte (`math::next_random`), the
//! time and total 0; and the race's copy of the player's settings (`dword_75AD00` ..
//! `dword_75AE24`: the co-driver, the race number, the view, the HUD's switches, the
//! steering), the HUD's bits (`dword_75B00C`) -1, `dword_75B108` 0.
//!
//! In:  eax = the player's name
//! Out: every register kept (`pusha` .. `popa`)
//! Callers: `net::start_link`, `net::open_lobby`. Its calls through `part`
//! (`net::lobby_calls`: the Rust versions). Shadow mode runs the original alone (the random
//! numbers). Self-test: a copy whose calls go to stand-ins, against the Rust version, with
//! a stage and with none.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    COPY_PRINTABLE, NEXT_RANDOM, OWN, OWN_TIME, OWN_TOTAL, PLAYER_NAME, Part, Saved, a, answer,
    dispatch_lobby, dword, fill, g, p, pb, region, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::fill_own_player", rva(0x4181A3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const STAGE: usize = 0x75ACD4; // dword_75ACD4: the stage chosen
pub const DIRECTION: usize = 0x75ACF0; // dword_75ACF0
const CAR: usize = 0x755A54; // g_player1.car_number
const COLOURS: usize = 0x755A7C; // g_player1.field_30
const RACE_NUMBER: usize = 0x755A74; // g_player1.race_number
const NATIONALITY: usize = 0x755A50; // g_player1.nationality

/// The settings copied: (to, from).
const SETTINGS: [(usize, usize); 13] = [
    (0x75AE20, 0x755BA0), // codriver_sex
    (0x75AE1C, 0x755A74), // race_number
    (0x75AE24, 0x755A58), // view
    (0x75ADF8, 0x755BA4), // map_area
    (0x75ADFC, 0x755BA8), // rear_mirror
    (0x75AE00, 0x755BAC), // rev_counter
    (0x75AE08, 0x755BB4), // message_display
    (0x75AE0C, 0x755BB0), // car_info
    (0x75AE14, 0x755BBC), // driver_info
    (0x75AE10, 0x755BB8), // progress_bar
    (0x75AE18, 0x755BC0), // pace_notes
    (0x75AE04, 0x755BC4), // metric_units
    (0x75AD2C, 0x755B68), // steering_sensitivity
];
/// Copied after the HUD's bits: (to, from).
const MORE: [(usize, usize); 5] = [
    (0x75AD00, 0x755650),
    (0x75AD0C, 0x75565C),
    (0x75AD10, 0x755660),
    (0x75AD14, 0x755664),
    (0x75AD18, 0x755668),
];
pub const HUD_BITS: usize = 0x75B00C; // dword_75B00C
const LAST: usize = 0x75B108; // dword_75B108

/// The Rust version: the registers it leaves (the entry's).
pub fn fill_own_player(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let mut c = Regs {
        ecx: 0x1F,
        ebx: a(PLAYER_NAME),
        ..r
    };
    part(COPY_PRINTABLE, &mut c)?;
    pb(OWN, 0);
    pb(OWN + 1, g(CAR) as u8);
    pb(OWN + 2, g(COLOURS) as u8);
    pb(OWN + 6, g(RACE_NUMBER) as u8);
    pb(OWN + 3, g(NATIONALITY) as u8);
    pb(OWN + 4, g(STAGE) as u8);
    pb(OWN + 5, g(DIRECTION) as u8);
    if crate::net::lobby::gb(OWN + 4) == 0 {
        pb(OWN + 4, 0x29);
        pb(OWN + 5, 0);
    }
    r.eax = g(DIRECTION);
    part(NEXT_RANDOM, &mut r)?;
    pb(OWN + 7, r.eax as u8);
    r.eax = 0;
    p(OWN_TIME, 0);
    p(OWN_TOTAL, 0);
    for (to, from) in SETTINGS {
        r.eax = g(from);
        p(to, r.eax);
    }
    p(HUD_BITS, u32::MAX);
    for (to, from) in MORE {
        r.eax = g(from);
        p(to, r.eax);
    }
    p(LAST, 0);
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, fill_own_player);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; pusha; mov ecx, 1Fh
    let copy = match crate::net::lobby::copy(
        0x4181A3,
        0x14F,
        &[0x60, 0x60, 0xB9, 0x1F, 0x00, 0x00, 0x00],
        &[0xF, 0x6F],
        &[COPY_PRINTABLE, NEXT_RANDOM],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [
        region(OWN, 0x10),
        region(0x75AD00, 0x410),
        dword(STAGE),
        dword(DIRECTION),
    ];
    let saved = Saved::new(&all);
    for stage in [0u32, 7, 0x1234_5600] {
        run_both(
            &mut t,
            &format!("stage {stage:#x}"),
            copy.addr(),
            Regs {
                eax: 0x0075_1234,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                fill(OWN, 0x10, 31);
                fill(0x75AD00, 0x410, 32);
                p(STAGE, stage);
                p(DIRECTION, 0x101);
                answer(Box::new(|place, r| {
                    if place == NEXT_RANDOM {
                        r.eax = 0x1234_5678;
                    }
                }));
            },
            &fill_own_player,
        );
    }
    drop(saved);
    t.finish()
}
