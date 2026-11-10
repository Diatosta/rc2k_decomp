//! `sound::car_sounds_all` (0x40D7AB): the turn's sounds of every car: the players' cars
//! each, and of the computer's only the two nearest the listener.
//!
//! Unless `dword_514F7C` (the objects in use) is 0: for each object whose +8 is 1, a
//! player's (+0 not negative) gets `sound::car_sounds` (eax its number) at once; of the
//! others the squared distance (+0x2C .. +0x34 squared and added, through `flt_514F88`)
//! is compared as an integer (signed, the bits of a float that is not negative) with
//! the nearest and the next (both starting at the bits of +inf), which keep their
//! numbers in eax and ebx (-1 for none). When the nearest is last turn's second
//! (`dword_514F84`) or the next last turn's nearest (`dword_514F80`), the two swap, so
//! that each keeps its voice; they are kept in the two dwords and each gets
//! `sound::car_sounds` with its +0 set to -2 (the nearest) or -1 for the call and put
//! back after. One caller, `sound::race_turn`.
//! Refuses (before anything) a count over 8 and a null object; what the callees refuse
//! puts back what was written (sounds already sent stay sent).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87_float::{ld, st};
use crate::sound::store::SoundObject;

/// Every car's sounds for the turn: the players' each, of the computer's the two nearest
/// the listener (each keeping its voice).
pub fn car_sounds_all(sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    let count = sound.race.sounds.count.get();
    if count == 0 {
        return Ok(());
    }
    if count > 8 {
        return Err(BadInput("a sound object count over 8").into());
    }
    // Converted: `rc2k_core::sound::turn::Nearest` (the choice; `car_sounds` keeps the
    // registers it is held in).
    use rc2k_core::sound::turn::{NEAREST_MARK, NEXT_MARK, Nearest};
    let mut nearest = Nearest::default();
    for n in 0..count {
        let o = SoundObject::number(n)?;
        if o.in_use.get() != 1 {
            continue;
        }
        if o.sound_offset.get() as i32 >= 0 {
            crate::sound::car_sounds::car_sounds(n, sound)?;
            continue;
        }
        x87_1(o, &sound.race.sounds);
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
        let o = SoundObject::number(n)?;
        let kept = o.sound_offset.get();
        o.sound_offset.set(mark);
        crate::sound::car_sounds::car_sounds(n, sound)?;
        o.sound_offset.set(kept);
    }
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `nearest_distance = (place[0] * place[0]) + ((place[1] * place[1]) + (place[2] * place[2]))`
fn x87_1(si: &SoundObject, df: &crate::sound::store::RaceSounds) {
    let t0 = ld(si.place[0].bits());
    let t1 = ld(si.place[0].bits());
    let t2 = t0 * t1;
    let t3 = ld(si.place[1].bits());
    let t4 = ld(si.place[1].bits());
    let t5 = t3 * t4;
    let t6 = ld(si.place[2].bits());
    let t7 = ld(si.place[2].bits());
    let t8 = t6 * t7;
    let t9 = t5 + t8;
    let t10 = t2 + t9;
    df.nearest_distance.set_bits(st(t10));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(si, df) "fld dword ptr [{si} + 0x2C]", "fmul dword ptr [{si} + 0x2C]", "fld dword ptr [{si} + 0x30]", "fmul dword ptr [{si} + 0x30]", "fld dword ptr [{si} + 0x34]", "fmul dword ptr [{si} + 0x34]", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fstp dword ptr [{df} + 0x2578]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "car_sounds_all line 70",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0x2C, In::F), (0x30, In::F), (0x34, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0x2578, Out::F)],
                },
            ],
            ulps: 0,
            reference: |a| unsafe { ref_0(a[0], a[1]) },
            plain: |a| {
                x87_1(
                    unsafe { &*(crate::pointers::host_at(a[0]) as *const SoundObject) },
                    unsafe {
                        &*(crate::pointers::host_at(a[1]) as *const crate::sound::store::RaceSounds)
                    },
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
