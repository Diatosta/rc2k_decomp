//! The shared spots' voices (stream B, 2026-10-18): `unk_5168C0`, 0x48 voices of 0x50
//! bytes, which `sound::spot_sounds` (0x40E611) keeps in step with the spots the race
//! frame shares each frame (`g_sharedSoundSpots`, 0x14 bytes: +0 an id, +4 .. +0xC the
//! place, +0x10 the kind 1 .. 0x15; `race::share_sound_spots`).
//!
//! A voice by offset (names by use): +0 the spot's id (0x1888FFFF: free), +4 heard this
//! turn, +8 .. +0x10 the place, +0x14 the sound (`dword_517F54[kind]`; -2 freed), +0x18
//! the kind, +0x1C 1 << kind, +0x20 new, +0x24 (re)start it, +0x28 playing, +0x2C the
//! repeats, +0x30 the delay before the next, +0x34 when it ends, +0x38 the rate and
//! +0x3C the volume (floats), +0x40 the distance, +0x44 its change, +0x48 and +0x4C the
//! pitch and volume factors.

use crate::framework::{game_addr, game_read, game_write, rva};
use crate::service::harness::{Region, at};

/// unk_5168C0.
pub const VOICES: usize = 0x5168C0;
pub const VOICE_LEN: u32 = 0x50;
pub const VOICE_COUNT: u32 = 0x48;
pub const FREE: u32 = 0x1888_FFFF;
/// g_sharedSoundSpots, g_sharedSoundSpotCount.
pub const SPOTS: usize = 0x5165F0;
pub const SPOT_COUNT: usize = 0x5165EC;
pub const SPOT_LEN: u32 = 0x14;
pub const MOST_SPOTS: u32 = 36;
/// dword_517F54: a kind's sound, 22 of them.
pub const SOUND_OF_KIND: usize = 0x517F54;
/// unk_517FAC: a kind's repeats (8 bytes, kind 1 first: +2 the most, +4 and +6 the
/// least and most delay, words).
pub const REPEATS: usize = 0x517FAC;
pub const MOST_KIND: u32 = 0x15;
/// dword_517F40 .. dword_517F50: the work of the two pickers (the playing one, the
/// nearest's distance and voice, the next's distance and voice).
pub const WORK: usize = 0x517F40;

/// The address of voice `n`.
pub fn voice(n: u32) -> u32 {
    game_addr(rva(VOICES)) as u32 + n * VOICE_LEN
}

/// The dword at `at` of the voice at `v`.
pub fn vw(v: u32, at: u32) -> u32 {
    unsafe { ((v + at) as *const u32).read_unaligned() }
}

pub fn set(v: u32, at: u32, value: u32) {
    unsafe { ((v + at) as *mut u32).write_unaligned(value) }
}

pub fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

pub fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The voices, the pickers' work and the scratch dword.
pub fn voice_regions() -> Vec<Region> {
    vec![
        (voice(0), (VOICE_LEN * VOICE_COUNT) as usize),
        (game_addr(rva(WORK)) as u32, 0x14),
        at(rva(crate::sound::turn_parts::SCRATCH)),
    ]
}

/// The voices filled for a test: a pattern, then `each` for every voice.
pub fn fill_voices(seed: u32, each: &dyn Fn(u32, u32)) {
    for n in 0..VOICE_COUNT {
        let v = voice(n);
        for k in 0..VOICE_LEN / 4 {
            set(
                v,
                4 * k,
                (n.wrapping_mul(0x9E37_79B9) ^ seed ^ (k * 0x0101_0101)) & 0x3F7F_FFFF,
            );
        }
        each(n, v);
    }
}
