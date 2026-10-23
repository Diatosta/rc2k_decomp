//! `sound::stage_sounds` (0x40E2A9): the stage's own sounds for the turn: three one-off
//! sounds, a timed one, two that play all the time and two at a random pitch and place.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! esi the request, its pitch and volume factors (+0x20, +0x24) 1.0. Each of
//! `dword_51659C`, `dword_5165A0`, `dword_5165A4` at 1 is cleared and sound 0x1AB,
//! 0x1AC, 0x1AD played (`sound::play_sound`: ebx and edx -0x6F, ecx and edi 0, ebp
//! -1.0). `dword_5165A8` at 1 (cleared) arms the timed sound: due (`dword_5165C4`) at
//! the clock + `dword_5165B8`, waiting (`dword_5165C8`) 1. While it waits and is not due
//! (`js`) the slot is skipped; once due, its slot (`unk_5165CC`) is told to start and the
//! wait ends. Otherwise the slot (`sound::play_timed_slot`) at the place
//! `dword_5165AC` .. `dword_5165B4`. Then sound 0x1C5 with the volume `dword_5165BC` / 2
//! (`sar`), edi 1.0, ebp -1.0, and 0x1C6 with `dword_5165C0`, edi 1.0, ebp 0. Each of
//! `dword_5165E4`, `dword_5165E8` at 1 (cleared): one `math::next_random` gives the
//! volume (0x21 + its low five bits), the rate (the mix rate + its next nine bits less
//! 0x100 times the mix rate over 4096) and two floats of nine bits less 0x100 and nine
//! bits, x 0.01 (through `g_soundScratch`), in ecx and edi, ebp 1.0: sound 0x1AE / 0x1AF.
//! One caller, `sound::race_turn`.
//! Refuses nothing of its own (what the slot refuses puts back what was written; sounds
//! already sent stay sent).
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls go to stand-ins (the random numbers a sequence of the
//! test's), against the Rust version.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::sound::sample_settings::OWN;
use crate::sound::turn_parts::{CLOCK, REQUEST, SCRATCH};

const ONE_OFFS: [(usize, u32); 3] = [(0x51659C, 0x1AB), (0x5165A0, 0x1AC), (0x5165A4, 0x1AD)];
const ARM: usize = 0x5165A8;
const PLACE: usize = 0x5165AC;
const DELAY: usize = 0x5165B8;
const VOLUME_A: usize = 0x5165BC;
const VOLUME_B: usize = 0x5165C0;
const DUE: usize = 0x5165C4;
const WAITING: usize = 0x5165C8;
const TIMED_SLOT: usize = 0x5165CC;
const RANDOMS: [(usize, u32); 2] = [(0x5165E4, 0x1AE), (0x5165E8, 0x1AF)];
/// g_mixRateSetting.
const MIX_RATE: usize = 0x4F82D4;
const ONE: u32 = 0x3F80_0000;
const MINUS_ONE: u32 = 0xBF80_0000;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

// ---- Rust version ----

/// The stage's own sounds for the turn.
pub fn stage_sounds() -> Result<(), Refused> {
    use crate::sound::turn_parts::play;
    let request = crate::sound::turn_parts::request();
    p(REQUEST + 0x20, ONE);
    p(REQUEST + 0x24, ONE);
    for (flag, sound) in ONE_OFFS {
        if g(flag) == 1 {
            p(flag, 0);
            play(sound, OWN, OWN, [0, 0, MINUS_ONE]);
        }
    }
    // Converted: `rc2k_core::sound::stage::wait_turn`.
    let clock = g(CLOCK);
    let mut wait = rc2k_core::sound::stage::Wait {
        arm: g(ARM),
        delay: g(DELAY),
        due: g(DUE),
        waiting: g(WAITING),
        start: g(TIMED_SLOT + 4),
    };
    let slot = rc2k_core::sound::stage::wait_turn(&mut wait, clock);
    for (at, v) in [(ARM, wait.arm), (DUE, wait.due), (WAITING, wait.waiting)] {
        p(at, v);
    }
    if wait.start != g(TIMED_SLOT + 4) {
        p(TIMED_SLOT + 4, wait.start);
    }
    if slot {
        for k in 0..3 {
            p(REQUEST + 0xC + 4 * k, g(PLACE + 4 * k));
        }
        let timed = game_addr(rva(TIMED_SLOT)) as u32;
        crate::sound::play_timed_slot::play_timed_slot(timed, request)?;
    }
    play(
        0x1C5,
        ((g(VOLUME_A) as i32) >> 1) as u32,
        OWN,
        [0, ONE, MINUS_ONE],
    );
    play(0x1C6, g(VOLUME_B), OWN, [0, ONE, 0]);
    for (flag, sound) in RANDOMS {
        if g(flag) != 1 {
            continue;
        }
        p(flag, 0);
        let n = unsafe { crate::math::next_random::next_random() };
        // Converted: `rc2k_core::sound::stage::random`.
        let r = rc2k_core::sound::stage::random(n, g(MIX_RATE));
        // The scratch dword holds the last float.
        p(SCRATCH, r.ahead.to_bits());
        play(
            sound,
            r.volume,
            r.rate,
            [r.across.to_bits(), r.ahead.to_bits(), ONE],
        );
    }
    Ok(())
}
