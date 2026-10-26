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
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::physics::flow::guard;
use crate::sound::car_slot_sounds::SLOT;
use crate::sound::sample_settings::OWN;
use crate::sound::turn_parts::{
    CLOCK, PLAY, Part, RANDOM, REQUEST, SCRATCH, common_regions, turn_entry,
};

/// The stage sounds' variables, `dword_51659C` .. `dword_5165E8`.
const VARS: usize = 0x51659C;
const VARS_LEN: usize = 0x50;
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

pub fn stage_sounds_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let mut all = common_regions();
    all.push((game_addr(rva(VARS)) as u32, VARS_LEN));
    guard(&all, || {
        let mut c = Regs {
            esi: game_addr(rva(REQUEST)) as u32,
            ..*r
        };
        p(REQUEST + 0x20, ONE);
        p(REQUEST + 0x24, ONE);
        for (flag, sound) in ONE_OFFS {
            if g(flag) == 1 {
                p(flag, 0);
                (c.eax, c.ebx, c.edx, c.ecx, c.edi, c.ebp) = (sound, OWN, OWN, 0, 0, MINUS_ONE);
                part(PLAY, &mut c)?;
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
        let (armed, waited) = (wait.arm == 1, wait.arm == 1 || wait.waiting == 1);
        let slot = rc2k_core::sound::stage::wait_turn(&mut wait, clock);
        if armed {
            c.eax = wait.due;
        }
        if waited {
            c.eax = clock;
        }
        for (at, v) in [(ARM, wait.arm), (DUE, wait.due), (WAITING, wait.waiting)] {
            p(at, v);
        }
        if wait.start != g(TIMED_SLOT + 4) {
            p(TIMED_SLOT + 4, wait.start);
        }
        if slot {
            for k in 0..3 {
                c.eax = g(PLACE + 4 * k);
                p(REQUEST + 0xC + 4 * k, c.eax);
            }
            c.ebp = game_addr(rva(TIMED_SLOT)) as u32;
            part(SLOT, &mut c)?;
        }
        (c.eax, c.ebx, c.edx, c.ecx, c.edi, c.ebp) = (
            0x1C5,
            ((g(VOLUME_A) as i32) >> 1) as u32,
            OWN,
            0,
            ONE,
            MINUS_ONE,
        );
        part(PLAY, &mut c)?;
        (c.eax, c.ebx, c.edx, c.ecx, c.edi, c.ebp) = (0x1C6, g(VOLUME_B), OWN, 0, ONE, 0);
        part(PLAY, &mut c)?;
        for (flag, sound) in RANDOMS {
            if g(flag) != 1 {
                continue;
            }
            p(flag, 0);
            part(RANDOM, &mut c)?;
            // Converted: `rc2k_core::sound::stage::random`.
            let n = c.eax;
            let r = rc2k_core::sound::stage::random(n, g(MIX_RATE));
            (c.ebx, c.edx) = (r.volume, r.rate);
            c.ecx = r.across.to_bits();
            c.eax = (n >> 23) & 0x1FF;
            c.edi = r.ahead.to_bits();
            // The scratch dword holds the last float.
            p(SCRATCH, c.edi);
            (c.ebp, c.eax) = (ONE, sound);
            part(PLAY, &mut c)?;
        }
        Ok(())
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, stage_sounds_in)
}
