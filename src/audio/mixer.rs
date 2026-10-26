//! The game's own mixer (the Dolby mode's way, `g_soundMode` bit 0x40000; IDA 2026-10-14):
//! the voices' channels mixed into a work buffer of left and right dwords
//! (`g_mixBuffer`, 0x952AF8, 0x3C00 bytes up to `g_soundCommands`; `MIX`, Rust's since
//! phase 12), then clipped into the primary buffer's format. The standalone program will
//! mix this way.

use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

pub const MIX_BUFFER_LEN: u32 = 0x3C00;

/// The mixing buffer (`g_mixBuffer`): a turn's left and right dwords, from `mix_channels`
/// to `write_mix`.
static MIX: std::sync::Mutex<[i32; MIX_BUFFER_LEN as usize / 4]> =
    std::sync::Mutex::new([0; MIX_BUFFER_LEN as usize / 4]);

/// `audio::mix_channels` (`sub_40BECC`): `bytes` of the primary buffer's worth, `>> shift`
/// frames, of every used channel (`count` from `channels`) added into `MIX` (zeroed
/// first, two dwords a frame: the sample times the channel's left and its right volume).
/// A channel's 32.32 position steps by its step after each frame; at its length a looping
/// channel (kind 1) goes back to 0 (the fraction kept), any other is no longer used and
/// stops there. Positions are stored back. Refuses no channels, no frames with a channel
/// used (both counts run 2^32 times in the original), more frames than the buffer holds.
pub fn mix_channels(bytes: u32, shift: u32, channels: u32, count: u32) -> Result<(), Refused> {
    use rc2k_core::audio::mixer::Channel;
    // Channel ids (place + 1) on from the first's.
    let at = |n: u32| channels.wrapping_add(n);
    // Converted: `rc2k_core::audio::mixer`; here the channels loaded and stored, the data read
    // where the channels point.
    let mut loaded: Vec<(Channel, u32)> = (0..count)
        .map(|n| {
            let c = channel(at(n));
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
    // The mix buffer worked on as a vector of its dwords (the core zeroes it first), written
    // back whole.
    let mut buffer = vec![0i32; MIX_BUFFER_LEN as usize / 4];
    // A channel's sample data (an id: `audio::sample_data`) found when the channel is first
    // read, all of it readable while the mix runs.
    let failed = std::cell::Cell::new(None);
    let mixed = crate::audio::sample_data::with_view(|view| {
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
    MIX.lock()
        .unwrap_or_else(|e| e.into_inner())
        .copy_from_slice(&buffer);
    for (n, core) in cores.iter().enumerate() {
        // Only the channels that played: the original stores no others.
        let c = channel(at(n as u32));
        if c.used == 1 {
            c.used = core.used;
            c.position = core.position;
            c.position_fraction = core.fraction;
        }
    }
    Ok(())
}

/// `audio::write_mix` (`sub_40C03A`, with `sub_40C087`, `sub_40C0E0`, `sub_40C153`): the
/// mix clipped into the two locked parts of buffer `primary` (`parts`: `platform_sound::
/// lock`'s; the second used if it has a place) in the primary buffer's format: eight
/// bits (mode bit 0x10) mono (0x100: left + right `>> 15`, held to -0x80 .. 0x7F, + 0x80),
/// eight bits stereo (each `>> 14`, the same), else sixteen bits stereo (each `>> 14`,
/// held to -0x8000 .. 0x7FFF). Refuses sixteen bits with a right sample that clips: the
/// original's clamp there goes back to the left sample's store and reads the same right
/// sample again, for ever.
pub fn write_mix(mode: u32, primary: u32, parts: [u32; 4]) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::mixer`; here the parts as slices of the locked buffer,
    // whole frames of each.
    let size = rc2k_core::audio::mixer::frame_bytes(mode);
    let mix = *MIX.lock().unwrap_or_else(|e| e.into_inner());
    let written = crate::platform_sound::with_parts(primary, parts, |first, second| {
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
