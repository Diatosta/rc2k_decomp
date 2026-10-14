//! The game's own mixer (the Dolby mode's way, `g_soundMode` bit 0x40000; IDA 2026-10-14):
//! the voices' channels mixed into a work buffer of left and right dwords
//! (`g_mixBuffer`, 0x952AF8, 0x3C00 bytes up to `g_soundCommands`), then clipped into the
//! primary buffer's format. The standalone program will mix this way.

use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::framework::rva;

pub const MIX_BUFFER_RVA: usize = rva(0x952AF8); // g_mixBuffer
pub const MIX_BUFFER_LEN: u32 = 0x3C00;
pub const MIX_RATE_RVA: usize = rva(0x5068D4); // g_mixRate
pub const PRIMARY_SIZE_RVA: usize = rva(0x5068D8); // g_primarySize
pub const MIX_MOST_RVA: usize = rva(0x5068DC); // g_mixMost: 40 ms of bytes
pub const MIX_SHIFT_RVA: usize = rva(0x5068E0); // g_mixShift: bytes to frames
pub const PLAY_CURSOR_RVA: usize = rva(0x5068E4); // g_playCursor
pub const WRITE_CURSOR_RVA: usize = rva(0x5068E8); // g_writeCursor

/// `audio::mix_channels` (`sub_40BECC`): `bytes` of the primary buffer's worth, `>> shift`
/// frames, of every used channel (`count` from `channels`) added into `mix` (zeroed
/// first, two dwords a frame: the sample times the channel's left and its right volume).
/// A channel's 32.32 position steps by its step after each frame; at its length a looping
/// channel (kind 1) goes back to 0 (the fraction kept), any other is no longer used and
/// stops there. Positions are stored back. Refuses no channels, no frames with a channel
/// used (both counts run 2^32 times in the original), more frames than the buffer holds.
pub unsafe fn mix_channels(
    bytes: u32,
    shift: u32,
    mix: u32,
    channels: u32,
    count: u32,
) -> Result<(), Refused> {
    if count == 0 {
        return Err(BadInput("no channels (the original's count runs 2^32 times)").into());
    }
    let frames = bytes >> (shift & 0x1F);
    if frames.wrapping_mul(8) > MIX_BUFFER_LEN || frames > MIX_BUFFER_LEN {
        return Err(BadInput("more frames than the mix buffer holds").into());
    }
    let size = std::mem::size_of::<MixerChannel>() as u32;
    unsafe {
        if frames == 0 && (0..count).any(|n| channel(channels + n * size).used == 1) {
            return Err(BadInput("no frames to mix (the original's count runs 2^32 times)").into());
        }
        std::ptr::write_bytes(mix as *mut u32, 0, frames as usize * 2);
        for n in 0..count {
            let c = channel(channels + n * size);
            if c.used != 1 {
                continue;
            }
            let (mut pos, mut fraction) = (c.position, c.position_fraction);
            let mut out = mix;
            let mut left_frames = frames;
            loop {
                let (l, r) = sample(c, pos);
                let at = out as *mut i32;
                at.write(at.read().wrapping_add(l.wrapping_mul(c.left)));
                let at = (out + 4) as *mut i32;
                at.write(at.read().wrapping_add(r.wrapping_mul(c.right)));
                let (f, carry) = fraction.overflowing_add(c.step_fraction);
                fraction = f;
                pos = pos.wrapping_add(c.step).wrapping_add(carry as u32);
                out += 8;
                if pos >= c.length {
                    if c.kind == 1 {
                        pos = 0;
                    } else {
                        c.used = 0;
                        break;
                    }
                }
                left_frames -= 1;
                if left_frames == 0 {
                    break;
                }
            }
            c.position = pos;
            c.position_fraction = fraction;
        }
    }
    Ok(())
}

/// The frame at `pos` of a channel's data: (left, right) before the volumes; a mono sample
/// is both.
unsafe fn sample(c: &MixerChannel, pos: u32) -> (i32, i32) {
    let data = c.data;
    let b = |at: u32| unsafe { *(at as *const u8) } as i32 - 0x80;
    let w = |at: u32| unsafe { (at as *const i16).read_unaligned() } as i32;
    let eight = c.format & crate::audio::samples::EIGHT_BITS != 0;
    let mono = c.format & crate::audio::samples::MONO != 0;
    match (eight, mono) {
        (true, true) => {
            let s = b(data.wrapping_add(pos));
            (s, s)
        }
        (true, false) => (
            b(data.wrapping_add(pos.wrapping_mul(2))),
            b(data.wrapping_add(pos.wrapping_mul(2)).wrapping_add(1)),
        ),
        (false, true) => {
            let s = w(data.wrapping_add(pos.wrapping_mul(2)));
            (s, s)
        }
        (false, false) => (
            w(data.wrapping_add(pos.wrapping_mul(4))),
            w(data.wrapping_add(pos.wrapping_mul(4)).wrapping_add(2)),
        ),
    }
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
    let eight = mode & crate::audio::samples::EIGHT_BITS != 0;
    let mono = mode & crate::audio::samples::MONO != 0;
    let shift = if !eight {
        2
    } else if mono {
        0
    } else {
        1
    };
    let frames = |bytes: u32| bytes >> shift;
    let mut total = frames(parts[1]);
    if parts[2] != 0 {
        total = total.wrapping_add(frames(parts[3]));
    }
    if total.wrapping_mul(8) > MIX_BUFFER_LEN || total > MIX_BUFFER_LEN {
        return Err(BadInput("more frames than the mix buffer holds").into());
    }
    let dw = |at: u32| unsafe { (at as *const i32).read() };
    if !eight {
        for n in 0..total {
            let right = dw(mix + n * 8 + 4) >> 14;
            if !(-0x8000..=0x7FFF).contains(&right) {
                return Err(BadInput(
                    "a sixteen-bit right sample that clips (the original loops for ever)",
                )
                .into());
            }
        }
    }
    unsafe {
        let mut from = mix;
        let second = if parts[2] != 0 {
            Some((parts[2], parts[3]))
        } else {
            None
        };
        for (to, bytes) in std::iter::once((parts[0], parts[1])).chain(second) {
            let mut to = to;
            for _ in 0..frames(bytes) {
                let (l, r) = (dw(from), dw(from + 4));
                from += 8;
                if !eight {
                    (to as *mut i16).write_unaligned((l >> 14).clamp(-0x8000, 0x7FFF) as i16);
                    ((to + 2) as *mut i16).write_unaligned((r >> 14) as i16);
                    to += 4;
                } else if mono {
                    let s = (l.wrapping_add(r) >> 15).clamp(-0x80, 0x7F);
                    *(to as *mut u8) = (s as u8).wrapping_add(0x80);
                    to += 1;
                } else {
                    *(to as *mut u8) = ((l >> 14).clamp(-0x80, 0x7F) as u8).wrapping_add(0x80);
                    *((to + 1) as *mut u8) =
                        ((r >> 14).clamp(-0x80, 0x7F) as u8).wrapping_add(0x80);
                    to += 2;
                }
            }
        }
    }
    Ok(())
}
