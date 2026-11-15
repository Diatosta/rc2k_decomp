//! The game's own mixer (the Dolby mode's way): the voices' channels mixed into a work buffer of
//! left and right dwords (`AudioOwn::mix`), then clipped into the primary buffer's format.

use crate::audio::own::AudioOwn;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

pub const MIX_BUFFER_LEN: u32 = 0x3C00;

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
    let mut loaded: Vec<(Channel, u32)> = (0..count)
        .map(|n| {
            let c = channel(state, at(n));
            let core = Channel {
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
            };
            (core, c.data)
        })
        .collect();
    let data: Vec<u32> = loaded.iter().map(|(_, d)| *d).collect();
    let mut cores: Vec<Channel> = loaded.drain(..).map(|(c, _)| c).collect();
    // The mix buffer worked on as a vector of its dwords (the core zeroes it first), written back
    // whole.
    let mut buffer = vec![0i32; MIX_BUFFER_LEN as usize / 4];
    // A channel's sample data (an id: `audio::sample_data`) found when the channel is first read,
    // all of it readable while the mix runs.
    let failed = std::cell::Cell::new(None);
    let mixed = state.sample_data.with_view(|view| {
        let samples: Vec<std::cell::OnceCell<Result<&[u8], BadInput>>> =
            data.iter().map(|_| std::cell::OnceCell::new()).collect();
        let byte = |n: usize, offset: u32| {
            let bytes = samples[n]
                .get_or_init(|| view.bytes(data[n]))
                .as_ref()
                .map_err(|e| *e)
                .and_then(|b| {
                    b.get(offset as usize)
                        .copied()
                        .ok_or(BadInput("a channel read past its sample data"))
                });
            bytes.unwrap_or_else(|e| {
                failed.set(failed.get().or(Some(e)));
                0
            })
        };
        rc2k_core::audio::mixer::mix_channels(
            bytes >> (shift & 0x1F),
            &mut cores,
            &byte,
            &mut buffer,
        )
    });
    if let Some(e) = failed.get() {
        return Err(e.into());
    }
    mixed?;
    state
        .mix
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .copy_from_slice(&buffer);
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
