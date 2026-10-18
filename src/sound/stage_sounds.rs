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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::physics::flow::{base, guard, x87_run};
use crate::service::harness::{Region, at, capture, restore};
use crate::sound::car_slot_sounds::SLOT;
use crate::sound::sample_settings::OWN;
use crate::sound::turn_parts::{
    CLOCK, PLAY, Part, RANDOM, REQUEST, SCRATCH, both, common_regions, copy_of, dispatch_turn,
    run_copy,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::stage_sounds", rva(0x40E2A9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x2F5;
const OUTS: [usize; 10] = [
    0x46, 0x7C, 0xB2, 0x12C, 0x152, 0x176, 0x192, 0x232, 0x24E, 0x2EE,
];
const ENTRY: [u8; 7] = [0x60, 0x8D, 0x35, 0xBC, 0x24, 0x51, 0x00];
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

/// `mov scratch, v; fild scratch; mov scratch, 0.01; fmul scratch; fstp scratch`.
fn hundredth(v: u32) -> u32 {
    p(SCRATCH, v);
    x87_run!(
        "fild dword ptr [{b} + 0x1183CC]",
        "mov dword ptr [{b} + 0x1183CC], 0x3C23D70A",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{b} + 0x1183CC]";
        b = base()
    );
    g(SCRATCH)
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
        let clock = g(CLOCK);
        if g(ARM) == 1 {
            p(ARM, 0);
            c.eax = clock.wrapping_add(g(DELAY));
            p(DUE, c.eax);
            p(WAITING, 1);
        }
        let mut slot = true;
        if g(WAITING) == 1 {
            c.eax = clock;
            if (clock.wrapping_sub(g(DUE)) as i32) < 0 {
                slot = false;
            } else {
                p(TIMED_SLOT + 4, 1);
                p(WAITING, 0);
            }
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
            let n = c.eax;
            c.ebx = (n & 0x1F) + 0x21;
            let mix = g(MIX_RATE);
            c.edx = ((((n >> 5) & 0x1FF).wrapping_sub(0x100) as i32).wrapping_mul(mix as i32) >> 12)
                as u32;
            c.edx = c.edx.wrapping_add(mix);
            c.ecx = hundredth(((n >> 14) & 0x1FF).wrapping_sub(0x100));
            c.eax = (n >> 23) & 0x1FF;
            c.edi = hundredth(c.eax);
            (c.ebp, c.eax) = (ONE, sound);
            part(PLAY, &mut c)?;
        }
        Ok(())
    })
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, stage_sounds_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40E2A9, LEN, &ENTRY, &OUTS, &[PLAY, SLOT, RANDOM]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut all: Vec<Region> = common_regions();
    all.extend([(a(VARS), VARS_LEN), at(rva(CLOCK)), at(rva(MIX_RATE))]);
    let saved = capture(&all);
    for case in 0..48u32 {
        let fill = || unsafe {
            let v = a(VARS) as *mut u32;
            for n in 0..VARS_LEN / 4 {
                v.add(n).write(0x40 + n as u32 * 3);
            }
            for (k, (flag, _)) in ONE_OFFS.iter().enumerate() {
                p(*flag, (case >> k) & 1);
            }
            p(ARM, (case >> 3) & 1);
            p(WAITING, [0, 1, 1][(case % 3) as usize]);
            p(DUE, [0, 5000, 0x8000_0000][(case % 3) as usize]);
            for (k, (flag, _)) in RANDOMS.iter().enumerate() {
                p(*flag, ((case >> 4) + k as u32) & 1);
            }
            p(VOLUME_A, 0xFFFF_FFF1);
            p(CLOCK, 1000);
            p(MIX_RATE, [22050, 44100, 11025][(case % 3) as usize]);
        };
        both(
            &mut t,
            &format!("case {case}"),
            &run,
            Regs::SENTINEL,
            &fill,
            &all,
            stage_sounds_in,
            false,
        );
    }
    restore(&all, &saved);
    t.finish()
}
