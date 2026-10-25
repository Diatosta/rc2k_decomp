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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::session::MACHINE;
use crate::sound::sample_settings::OWN;
use crate::sound::store::STAGE_SOUNDS;
use crate::sound::turn_parts::REQUEST;

/// The one-off sounds: the flag's index in `once` and the sound.
const ONE_OFFS: [(usize, u32); 3] = [(0, 0x1AB), (1, 0x1AC), (2, 0x1AD)];
/// The random sounds: the flag's index in `random` and the sound.
const RANDOMS: [(usize, u32); 2] = [(0, 0x1AE), (1, 0x1AF)];
/// g_mixRateSetting.
const ONE: u32 = 0x3F80_0000;
const MINUS_ONE: u32 = 0xBF80_0000;

fn p(x: usize, v: u32) {
    unsafe { crate::store::ida_write(x, v) }
}

// ---- Rust version ----

/// The stage's own sounds for the turn.
pub fn stage_sounds() -> Result<(), Refused> {
    use crate::sound::turn_parts::play;
    let request = crate::sound::turn_parts::request();
    let stage = &STAGE_SOUNDS;
    p(REQUEST + 0x20, ONE);
    p(REQUEST + 0x24, ONE);
    for (n, sound) in ONE_OFFS {
        if stage.once[n].get() == 1 {
            stage.once[n].set(0);
            play(sound, OWN, OWN, [0, 0, MINUS_ONE]);
        }
    }
    // Converted: `rc2k_core::sound::stage::wait_turn`.
    let clock = crate::sound::store::RACE_CLOCK.last.get();
    let mut wait = rc2k_core::sound::stage::Wait {
        arm: stage.timed_arm.get(),
        delay: stage.timed_delay.get(),
        due: stage.timed_due.get(),
        waiting: stage.timed_waiting.get(),
        start: stage.timed_slot.start.get(),
    };
    let slot = rc2k_core::sound::stage::wait_turn(&mut wait, clock);
    stage.timed_arm.set(wait.arm);
    stage.timed_due.set(wait.due);
    stage.timed_waiting.set(wait.waiting);
    if wait.start != stage.timed_slot.start.get() {
        stage.timed_slot.start.set(wait.start);
    }
    if slot {
        for k in 0..3 {
            p(REQUEST + 0xC + 4 * k, stage.timed_place[k].bits());
        }
        let timed = &stage.timed_slot as *const _ as usize as u32;
        crate::sound::play_timed_slot::play_timed_slot(timed, request)?;
    }
    play(
        0x1C5,
        ((stage.loop_volume_a.get() as i32) >> 1) as u32,
        OWN,
        [0, ONE, MINUS_ONE],
    );
    play(0x1C6, stage.loop_volume_b.get(), OWN, [0, ONE, 0]);
    for (n, sound) in RANDOMS {
        if stage.random[n].get() != 1 {
            continue;
        }
        stage.random[n].set(0);
        let n = unsafe { crate::math::next_random::next_random() };
        // Converted: `rc2k_core::sound::stage::random`.
        let r =
            rc2k_core::sound::stage::random(n, crate::audio::store::AUDIO.mix_rate_setting.get());
        // The scratch dword holds the last float.
        MACHINE.scratch.set(r.ahead.to_bits());
        play(
            sound,
            r.volume,
            r.rate,
            [r.across.to_bits(), r.ahead.to_bits(), ONE],
        );
    }
    Ok(())
}
