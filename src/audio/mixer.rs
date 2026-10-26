//! The game's own mixer (the Dolby mode's way, `g_soundMode` bit 0x40000; IDA 2026-10-14):
//! the voices' channels mixed into a work buffer of left and right dwords
//! (`g_mixBuffer`, 0x952AF8, 0x3C00 bytes up to `g_soundCommands`), then clipped into the
//! primary buffer's format. The standalone program will mix this way.

use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use rc2k_core::strings::bytes::Bytes as _;

pub const MIX_BUFFER_LEN: u32 = 0x3C00;

/// `audio::mix_channels` (`sub_40BECC`): `bytes` of the primary buffer's worth, `>> shift`
/// frames, of every used channel (`count` from `channels`) added into `mix` (zeroed
/// first, two dwords a frame: the sample times the channel's left and its right volume).
/// A channel's 32.32 position steps by its step after each frame; at its length a looping
/// channel (kind 1) goes back to 0 (the fraction kept), any other is no longer used and
/// stops there. Positions are stored back. Refuses no channels, no frames with a channel
/// used (both counts run 2^32 times in the original), more frames than the buffer holds.
pub fn mix_channels(
    bytes: u32,
    shift: u32,
    mix: u32,
    channels: u32,
    count: u32,
) -> Result<(), Refused> {
    use rc2k_core::audio::mixer::Channel;
    let size = std::mem::size_of::<MixerChannel>() as u32;
    let at = |n: u32| channels.wrapping_add(n.wrapping_mul(size));
    // Converted: `rc2k_core::audio::mixer`; here the channels loaded and stored, the data read
    // where the channels point.
    let mut loaded: Vec<(Channel, u32)> = (0..count)
        .map(|n| {
            let c = { channel(at(n)) };
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
    // back whole; a channel's sample data resolved when the channel is first read.
    let mut buffer = vec![0i32; MIX_BUFFER_LEN as usize / 4];
    let samples: Vec<std::cell::OnceCell<Result<crate::pointers::GameBytes, BadInput>>> =
        data.iter().map(|_| std::cell::OnceCell::new()).collect();
    let failed = std::cell::Cell::new(None);
    let byte = |n: usize, offset: u32| {
        let bytes = samples[n]
            .get_or_init(|| GamePtr::new(data[n]).bytes("a channel's sample data"))
            .as_ref()
            .map_err(|e| *e)
            .and_then(|b| {
                b.byte(offset as usize)
                    .ok_or(BadInput("a channel read past its sample data"))
            });
        bytes.unwrap_or_else(|e| {
            failed.set(failed.get().or(Some(e)));
            0
        })
    };
    let mixed = rc2k_core::audio::mixer::mix_channels(
        bytes >> (shift & 0x1F),
        &mut cores,
        &byte,
        &mut buffer,
    );
    if let Some(e) = failed.get() {
        return Err(e.into());
    }
    mixed?;
    let out: Vec<u8> = buffer.iter().flat_map(|d| d.to_le_bytes()).collect();
    GamePtr::new(mix).write_bytes(&out, "the mix buffer")?;
    for (n, core) in cores.iter().enumerate() {
        // Only the channels that played: the original stores no others.
        let c = { channel(at(n as u32)) };
        if c.used == 1 {
            c.used = core.used;
            c.position = core.position;
            c.position_fraction = core.fraction;
        }
    }
    Ok(())
}

/// `audio::write_mix` (`sub_40C03A`, with `sub_40C087`, `sub_40C0E0`, `sub_40C153`): the
/// mix, from `mix` on, clipped into the two locked parts (`parts`: pointer and bytes,
/// twice; the second used if its pointer is not 0) in the primary buffer's format: eight
/// bits (mode bit 0x10) mono (0x100: left + right `>> 15`, held to -0x80 .. 0x7F, + 0x80),
/// eight bits stereo (each `>> 14`, the same), else sixteen bits stereo (each `>> 14`,
/// held to -0x8000 .. 0x7FFF). Refuses sixteen bits with a right sample that clips: the
/// original's clamp there goes back to the left sample's store and reads the same right
/// sample again, for ever.
pub unsafe fn write_mix(mode: u32, mix: u32, parts: [u32; 4]) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::mixer`; here the parts as slices of the locked buffer.
    let size = rc2k_core::audio::mixer::frame_bytes(mode) as u32;
    // The locked parts checked (the buffers' bytes are regions while they live).
    crate::race::trap::span(parts[0], (parts[1] / size * size) as usize, "a locked part")?;
    if parts[2] != 0 {
        crate::race::trap::span(parts[2], (parts[3] / size * size) as usize, "a locked part")?;
    }
    let mut slices: Vec<&mut [u8]> = vec![unsafe {
        std::slice::from_raw_parts_mut(
            crate::pointers::host(parts[0]),
            (parts[1] / size * size) as usize,
        )
    }];
    if parts[2] != 0 {
        slices.push(unsafe {
            std::slice::from_raw_parts_mut(
                crate::pointers::host(parts[2]),
                (parts[3] / size * size) as usize,
            )
        });
    }
    let buffer: Vec<i32> = GamePtr::new(mix)
        .read_bytes(MIX_BUFFER_LEN as usize, "the mix buffer")?
        .as_chunks::<4>()
        .0
        .iter()
        .map(|d| i32::from_le_bytes(*d))
        .collect();
    Ok(rc2k_core::audio::mixer::write_mix(
        mode,
        &buffer,
        &mut slices,
    )?)
}
