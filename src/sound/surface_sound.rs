//! `sound::surface_sound` (0x40E068): one of a car's sounds of what its wheels run on.
//!
//! Takes: the car's sound object, the surface's sound set, the request
//! The set: +0 1 for a set of eight (+4 ..; the sound by the object's +0 & 3 and, 4
//! further for +0xC 1), else four (by +0 & 3). Its sound replaces the object's +0x144
//! (sets of eight) or +0x148; a changed one has the old one stopped first
//! (`sound::stop_sound`). Then the request: the sound, +4 the rate as a float, +8
//! `SurfaceSounds.volume` (`fld` / `fstp`), the place +0x2C .. +0x34, and
//! `sound::play_placed_sound` (a tail jump). Two callers, `sound::surface_sounds`.
//! Refuses (before anything) a set of eight with +0xC
//! over 1, and a sound past the sample records.

use crate::errors::{BadInput, Refused};
use crate::math::x87::from_int;
use crate::math::x87_float::{ld, st};
use crate::pointers::GamePtr;
use crate::session::Cell32;
use crate::sound::sample_settings::{OWN, record, sample_settings};
use crate::sound::sets_store::SoundRequest;
use rc2k_core::sound::surfaces;

/// Object `o`'s sound of the surface's sound `set`, through `request`.
pub fn surface_sound(
    o: &crate::sound::store::SoundObject,
    set: GamePtr,
    request: &SoundRequest,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::surfaces::pick`.
    let eight = set.read::<u32>("a sound set")? == 1;
    let (n, slot) = surfaces::pick(eight, o.sound_offset.get(), || o.view_flag.get())
        .map_err(BadInput::from)?;
    // The core names the object's dword it replaces by its offset.
    let playing = match slot {
        0x144 => &o.eights_sound,
        0x148 => &o.fours_sound,
        _ => return Err(BadInput("a surface sound's slot not worked out").into()),
    };
    // The set is a list of dwords inside the sets' lists, read by position (no record of its own).
    let sound: u32 = set.offset(4 * (1 + n)).read("a sound set")?;
    record(sound)?;
    if sound != playing.get() {
        let old = playing.get();
        playing.set(sound);
        request.sound.set(old);
        crate::sound::turn_parts::stop(request)?;
    }
    let sound = playing.get();
    request.sound.set(sound);
    let (_, rate) = sample_settings(sound, OWN, OWN)?;
    request.pitch.set(from_int(rate as i32).to_bits());
    x87_1(&crate::sound::owner::sound().race.surface, &request.volume);
    for (cell, place) in request.place.iter().zip(&o.place) {
        cell.set(place.bits());
    }
    crate::sound::turn_parts::placed(request)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `volume = [sf + 0x40]` (the request's +8)
fn x87_1(sf: &crate::sound::store::SurfaceSounds, volume: &impl Cell32) {
    let t0 = ld(sf.volume.bits());
    volume.set_bits(st(t0));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::HostCell;

    // The original's instructions, one function per run.
    reference!(ref_0(sf, si) "fld dword ptr [{sf} + 0x40]", "fstp dword ptr [{si} + 0x8]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "surface_sound line 52",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0x40, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(8, Out::F)],
                },
            ],
            ulps: 0,
            reference: |a| unsafe { ref_0(a[0], a[1]) },
            plain: |a| {
                x87_1(
                    unsafe {
                        &*(crate::pointers::host_at(a[0])
                            as *const crate::sound::store::SurfaceSounds)
                    },
                    &HostCell(a[1] + 8),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
