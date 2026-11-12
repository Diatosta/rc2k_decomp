//! `sound::car_sounds_all` (0x40D7AB): the turn's sounds of every car: the players' cars
//! each, and of the computer's only the two nearest the listener.
//!
//! Unless `RaceSounds.count` (the objects in use) is 0: for each object whose +8 is 1, a
//! player's (+0 not negative) gets `sound::car_sounds` (with its number) at once; of the
//! others the squared distance (+0x2C .. +0x34 squared and added, through `RaceSounds.nearest_distance`)
//! is compared as an integer (signed, the bits of a float that is not negative) with
//! the nearest and the next (both starting at the bits of +inf), which keep their
//! numbers (-1 for none). When the nearest is last turn's second
//! (`RaceSounds.next`) or the next last turn's nearest (`RaceSounds.nearest`), the two swap, so
//! that each keeps its voice; they are kept in the two dwords and each gets
//! `sound::car_sounds` with its +0 set to -2 (the nearest) or -1 for the call and put
//! back after. One caller, `sound::race_turn`.
//! Refuses (before anything) a count over 8 and a null object; what the callees refuse
//! puts back what was written (sounds already sent stay sent).

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
    let count = sound.race.sounds.count.get();
    if count == 0 {
        return Ok(());
    }
    if count > 8 {
        return Err(BadInput("a sound object count over 8").into());
    }
    // Converted: `rc2k_core::sound::turn::Nearest` (the choice).
    use rc2k_core::sound::turn::{NEAREST_MARK, NEXT_MARK, Nearest};
    let mut nearest = Nearest::default();
    for n in 0..count {
        let o = SoundObject::of(sound, n)?;
        if o.in_use.get() != 1 {
            continue;
        }
        if o.sound_offset.get() as i32 >= 0 {
            crate::sound::car_sounds::car_sounds(n, sound, race, system, audio)?;
            continue;
        }
        x87_1(o, &sound.race.sounds.nearest_distance);
        nearest.consider(n, sound.race.sounds.nearest_distance.bits());
    }
    let (first, second) = nearest.kept(
        sound.race.sounds.nearest.get(),
        sound.race.sounds.next.get(),
    );
    sound.race.sounds.nearest.set(first);
    sound.race.sounds.next.set(second);
    for (which, mark) in [
        (&sound.race.sounds.nearest, NEAREST_MARK),
        (&sound.race.sounds.next, NEXT_MARK),
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

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `nearest_distance = (place[0] * place[0]) + ((place[1] * place[1]) + (place[2] * place[2]))`
fn x87_1(si: &SoundObject, nearest_distance: &impl crate::session::Cell32) {
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
    nearest_distance.set_bits(stf(t10).to_bits());
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::scene::x87_mem::HostCell;
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
                // The distance sits at +0x2578 of the race sounds in the image.
                with_laid::<SoundObject, _>(a[0], |so| x87_1(so, &HostCell(a[1] + 0x2578)));
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("sound.car_sounds_all", &blocks(), 1200, 100);
    }
}
