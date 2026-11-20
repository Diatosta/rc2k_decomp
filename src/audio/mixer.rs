//! The game's own mixer (the Dolby mode's way): the voices' channels mixed into a work buffer of
//! left and right dwords (`AudioOwn::mix`), then clipped into the primary buffer's format.

use crate::audio::own::AudioOwn;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

pub(super) const MIX_BUFFER_LEN: u32 = 0x3C00;

/// `bytes` of the primary buffer's worth, `>> shift` frames, of every used channel added into
/// the mix buffer (zeroed first, two dwords a frame: the sample times the channel's left and its right
/// volume). A channel's 32.32 position steps by its step after each frame; at its length a looping
/// channel (kind 1) goes back to 0 (the fraction kept), any other is no longer used and stops.
/// Refuses no channels, no frames with a channel used, and more frames than the buffer holds.
pub fn mix_channels(
    state: &AudioOwn,
    bytes: u32,
    shift: u32,
    channels: u32,
    count: u32,
) -> Result<(), Refused> {
    use rc2k_core::audio::mixer::Channel;
    // Channel ids (place + 1) on from the first's.
    let at = |n: u32| channels.wrapping_add(n);
    let mut scratch = state
        .mix_scratch
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let MixScratch { cores, data } = &mut *scratch;
    cores.clear();
    data.clear();
    for n in 0..count {
        let c = channel(state, at(n));
        cores.push(Channel {
            used: c.used,
            format: c.format,
            length: c.length,
            kind: c.kind,
            position: c.position,
            fraction: c.position_fraction,
            step: c.step,
            step_fraction: c.step_fraction,
            left: c.left,
            right: c.right,
        });
        data.push(c.data);
    }
    let failed = std::cell::Cell::new(None);
    // The mix buffer is mixed in place (the core zeroes what it fills; the rest is zeroed here).
    let mut buffer = state
        .mix
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let frames = (bytes >> (shift & 0x1F)) as usize;
    let mixed = state.sample_data.with_view(|view| {
        // A playing channel's sample data (an id: `audio::sample_data`) found once for the
        // turn, all of it readable while the mix runs.
        let samples: Vec<Result<&[u8], BadInput>> = cores
            .iter()
            .zip(data.iter())
            .map(|(core, id)| {
                if core.used == 1 {
                    view.bytes(*id)
                } else {
                    Ok(&[][..])
                }
            })
            .collect();
        let byte = |n: usize, offset: u32| {
            let found = samples[n].as_ref().map_err(|e| *e).and_then(|b| {
                b.get(offset as usize)
                    .copied()
                    .ok_or(BadInput("a channel read past its sample data"))
            });
            found.unwrap_or_else(|e| {
                failed.set(failed.get().or(Some(e)));
                0
            })
        };
        rc2k_core::audio::mixer::mix_channels(frames as u32, cores, &byte, &mut buffer[..])
    });
    if let Some(e) = failed.get() {
        return Err(e.into());
    }
    mixed?;
    if let Some(rest) = buffer.get_mut(frames * 2..) {
        rest.fill(0);
    }
    drop(buffer);
    for (n, core) in cores.iter().enumerate() {
        // Only the channels that played are stored back.
        update_channel(state, at(n as u32), |c| {
            if c.used == 1 {
                c.used = core.used;
                c.position = core.position;
                c.position_fraction = core.fraction;
            }
        });
    }
    Ok(())
}

/// The channel lists of a turn, kept so that a turn allocates none.
#[derive(Default)]
pub struct MixScratch {
    cores: Vec<rc2k_core::audio::mixer::Channel>,
    data: Vec<u32>,
}

/// Clips the mix into the two locked parts of buffer `primary` in the primary buffer's format:
/// Eight bits mono (left + right `>> 15`, held to -0x80 .. 0x7F, + 0x80), eight bits stereo (each
/// `>> 14`, the same), else sixteen bits stereo (each `>> 14`, held to -0x8000 .. 0x7FFF).
/// Refuses sixteen bits with a right sample that clips.
pub fn write_mix(
    state: &AudioOwn,
    mode: u32,
    primary: u32,
    parts: [u32; 4],
) -> Result<(), Refused> {
    let size = rc2k_core::audio::mixer::frame_bytes(mode);
    let mix = *state
        .mix
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let written = crate::platform::sound::with_parts(primary, parts, |first, second| {
        let whole = |p: &mut [u8]| p.len() / size * size;
        let n = whole(first);
        let mut slices: Vec<&mut [u8]> = vec![&mut first[..n]];
        if parts[2] != 0 {
            let n = whole(second);
            slices.push(&mut second[..n]);
        }
        rc2k_core::audio::mixer::write_mix(mode, &mix, &mut slices)
    })
    .map_err(|_| BadInput("a locked part that is not the buffer's"))?;
    Ok(written?)
}
