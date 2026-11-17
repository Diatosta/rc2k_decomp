//! `sound::car_sounds_all`: the turn's sounds of every car: the players' cars each, and of the
//! computer's only the two nearest the listener.
//!
//! Unless no object is in use: each object in use that belongs to a player gets `sound::car_sounds`
//! (with its number) at once. Of the others the squared distance (through
//! `RaceSounds.nearest_distance`) is compared as an integer (signed, the bits of a float that is
//! not negative) with the nearest and the next (both starting at the bits of +inf), which keep
//! their numbers (-1 for none). When the nearest is last turn's second (`RaceSounds.next`) or the
//! next last turn's nearest (`RaceSounds.nearest`), the two swap, so that each keeps its voice;
//! they are kept in the two dwords and each gets `sound::car_sounds` with its `sound_offset` set to
//! -2 (the nearest) or -1 for the call and put back after. Refuses (before anything) a count over 8
//! and a null object; what the callees refuse puts back what was written (sounds already sent stay
//! sent).

use crate::errors::{BadInput, Refused};
use crate::math::x87_float::{ld, stf};
use crate::sound::store::SoundObject;

/// Every car's sounds for the turn: the players' each, of the computer's the two nearest
/// the listener (each keeping its voice).
pub fn car_sounds_all(
    sound: &crate::sound::owner::Sound,
    race: &crate::race::owner::Race,
    system: &crate::system::owner::System,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    let count = sound.threads.shared.count.get();
    if count == 0 {
        return Ok(());
    }
    if count > 8 {
        return Err(BadInput("a sound object count over 8").into());
    }
    // The logic is `rc2k_core::sound::turn::Nearest` (the choice).
    use rc2k_core::sound::turn::{NEAREST_MARK, NEXT_MARK, Nearest};
    let mut nearest = Nearest::default();
    for n in 0..count {
        let o = SoundObject::of(sound, n)?;
        if o.in_use.get() != 1 {
            continue;
        }
        if o.sound_offset.get() as i32 >= 0 {
            // A player's car: its sounds go to the reverb of that player's view.
            crate::audio::reverb_views::playing_for(n as usize, || {
                crate::sound::car_sounds::car_sounds(n, sound, race, system, audio)
            })?;
            continue;
        }
        x87_1(o, &sound.race.sounds.get().nearest_distance);
        nearest.consider(n, sound.race.sounds.get().nearest_distance.get());
    }
    let (first, second) = nearest.kept(
        sound.race.sounds.get().nearest.get(),
        sound.race.sounds.get().next.get(),
    );
    sound.race.sounds.get().nearest.set(first);
    sound.race.sounds.get().next.set(second);
    for (which, mark) in [
        (&sound.race.sounds.get().nearest, NEAREST_MARK),
        (&sound.race.sounds.get().next, NEXT_MARK),
    ] {
        let n = which.get();
        if (n as i32) < 0 {
            continue;
        }
        let o = SoundObject::of(sound, n)?;
        let kept = o.sound_offset.get();
        o.sound_offset.set(mark);
        crate::sound::car_sounds::car_sounds(n, sound, race, system, audio)?;
        o.sound_offset.set(kept);
    }
    Ok(())
}

// ---- the x87-precision arithmetic ----

/// An x87-precision run in plain Rust over `X`, written as the expressions it computes
/// (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `nearest_distance = (place[0] * place[0]) + ((place[1] * place[1]) + (place[2] * place[2]))`
fn x87_1(si: &SoundObject, nearest_distance: &crate::memory::session::PlainFloat) {
    let t0 = ld(si.place[0].get());
    let t1 = ld(si.place[0].get());
    let t2 = t0 * t1;
    let t3 = ld(si.place[1].get());
    let t4 = ld(si.place[1].get());
    let t5 = t3 * t4;
    let t6 = ld(si.place[2].get());
    let t7 = ld(si.place[2].get());
    let t8 = t6 * t7;
    let t9 = t5 + t8;
    let t10 = t2 + t9;
    nearest_distance.set(stf(t10));
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::sound::asm_layout::{Laid, with_laid};
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "car_sounds_all line 70",
            operands: vec![
                Operand::Sized {
                    len: SoundObject::EXTENT,
                    reads: &[(0x2C, In::F), (0x30, In::F), (0x34, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0x2578, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                // The distance sits at +0x2578 of the race sounds' 1999 layout.
                with_laid::<SoundObject, _>(a[0], |so| {
                    x87_1(
                        so,
                        crate::scene::x87_mem::view::<crate::memory::session::PlainFloat>(
                            a[1], 0x2578,
                        ),
                    );
                });
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("sound.car_sounds_all", &blocks(), 1200, 100);
    }
}
