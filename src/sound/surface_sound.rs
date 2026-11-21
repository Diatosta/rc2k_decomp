//! `sound::surface_sound`: one of a car's sounds of what its wheels run on.
//!
//! Takes: the car's sound object, the surface's sound set, the request. The set: its first dword is 1 for a set of
//! eight (the sound picked by the object's `sound_offset` & 3 and, 4 further when `view_flag` is
//! 1), else four (by `sound_offset` & 3). Its sound replaces the object's `eights_sound` (sets of
//! eight) or `fours_sound`; a changed one has the old one stopped first (`sound::stop_sound`).
//! Then the request: the sound, the rate as a float (`pitch`), `SurfaceSounds.volume` as the
//! `volume`, the `place`, and `sound::play_placed_sound`. Refuses (before anything) a set of
//! eight with `view_flag` over 1, and a sound past the sample records.

use crate::errors::{BadInput, Refused};
use crate::math::x87::from_int;
use crate::math::x87_float::{ld, st};
use crate::sound::sample_settings::{OWN, record, sample_settings};
use crate::sound::sets_store::{ListIn, SoundRequest};
use rc2k_core::sound::surfaces;

/// Object `o`'s sound of the surface's sound `set`, through `request`.
pub fn surface_sound(
    o: &crate::sound::store::SoundObjectCell,
    set: ListIn<'_, 392>,
    request: &mut SoundRequest,
    owner: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    // The logic is `rc2k_core::sound::surfaces::pick`.
    let eight = set.get(0)? == 1;
    let (n, slot) = surfaces::pick(eight, o.borrow().sound_offset, || o.borrow().view_flag)
        .map_err(BadInput::from)?;
    // The core names which of the object's sounds it replaces by a slot number.
    let fours = match slot {
        0x144 => false,
        0x148 => true,
        _ => return Err(BadInput("a surface sound's slot not worked out").into()),
    };
    let playing = || {
        let object = o.borrow();
        if fours {
            object.fours_sound
        } else {
            object.eights_sound
        }
    };
    // The set is a list of dwords inside the sets' lists, read by position (no record of its own).
    let sound = set.get(1 + n as usize)?;
    record(audio, sound)?;
    let now = playing();
    if sound != now {
        let mut object = o.borrow_mut();
        let cell = if fours {
            &mut object.fours_sound
        } else {
            &mut object.eights_sound
        };
        *cell = sound;
        drop(object);
        request.sound = now;
        crate::sound::turn_parts::stop(request, audio)?;
    }
    let sound = playing();
    request.sound = sound;
    let (_, rate) = sample_settings(audio, sound, OWN, OWN)?;
    request.pitch = from_int(rate as i32).to_bits();
    x87_1(owner.race.surface.get(), &mut request.volume);
    request.place = o.borrow().place;
    crate::sound::turn_parts::placed(request, owner, audio)
}

// ---- the x87-precision arithmetic ----

/// An x87-precision run in plain Rust over `X`: the request's volume is the sound set's
/// `volume`.
fn x87_1(sf: &crate::sound::store::SurfaceSounds, volume: &mut u32) {
    let t0 = ld(sf.work.borrow().volume);
    *volume = st(t0);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::math::x87_mem::HostCell;
    use crate::memory::session::Cell32;
    use crate::sound::asm_layout::{SURFACE_EXTENT, with_surface};
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "surface_sound line 52",
            operands: vec![
                Operand::Sized {
                    len: SURFACE_EXTENT,
                    reads: &[(0x40, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                with_surface(a[0], |sf| {
                    let volume = HostCell(a[1] + 8);
                    let mut bits = volume.bits();
                    x87_1(sf, &mut bits);
                    volume.set_bits(bits);
                });
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("sound.surface_sound", &blocks(), 1200, 100);
    }
}
