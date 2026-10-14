//! `audio::mix_channels` (0x40BECC): the used channels mixed into the work buffer.
//!
//! In:  eax = the primary buffer's bytes to fill
//! Out: eax = the same (ebx the frames, ecx 0, edx, esi, edi, ebp the loops' leftovers:
//!      the one caller, `audio::mix`, keeps its own around it)
//!
//! See `audio/mixer.rs`. Compared in shadow mode (the work buffer and the channels).
//! Self-test: a redirected copy whose shift, buffer and channels are the test's, 120
//! channels' states by lot.

use crate::audio::mixer::*;
use crate::audio::mixer_harness::dispatch_written;
use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::mix_channels", rva(0x40BECC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub unsafe fn mix_channels_in_game(bytes: u32) -> Result<(), Refused> {
    let w = World::game();
    unsafe {
        mix_channels(
            bytes,
            game_read(MIX_SHIFT_RVA),
            game_addr(MIX_BUFFER_RVA) as u32,
            w.channels,
            read(w.channel_count),
        )
    }
}

fn rust(r: &mut Regs) -> Result<(), Refused> {
    unsafe { mix_channels_in_game(r.eax) }
}

extern "C" fn dispatch(r: &mut Regs) {
    let w = World::game();
    let count = unsafe { read(w.channel_count) }.min(0x40) as usize;
    let regions = [
        (game_addr(MIX_BUFFER_RVA) as u32, MIX_BUFFER_LEN as usize),
        (w.channels, count * std::mem::size_of::<MixerChannel>()),
    ];
    dispatch_written(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mix = vec![0u32; MIX_BUFFER_LEN as usize / 4];
    let mut channels = vec![MixerChannel::default(); 6];
    let mut vars = Box::new([0u32; 2]);
    let at = vars.as_mut_ptr() as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40BECC,
            len: 0x16E,
            entry: &[0x50, 0x8B, 0x0D, 0xE0, 0x68, 0x50, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x5068E0, at),
            (0x952AF8, mix.as_mut_ptr() as u32),
            (0x506354, channels.as_mut_ptr() as u32),
            (0x50634C, at + 4),
        ],
        5,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let data: Vec<u8> = (0..0x800u32)
        .map(|i| (i.wrapping_mul(0x9E37) >> 5) as u8)
        .collect();
    let mut rng = 0xBEEF_0001u32;
    for case in 0..120 {
        let seed = rng;
        let mut next = || {
            rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            rng >> 8
        };
        let shift = next() % 3;
        let bytes = ((next() % 0x200) + 1) << shift;
        let count = 1 + next() % 6;
        let setup: Vec<MixerChannel> = (0..6)
            .map(|_| MixerChannel {
                used: [1, 1, 0, 2][next() as usize % 4],
                format: [0x110u32, 0x210, 0x120, 0x220][next() as usize % 4],
                data: data.as_ptr() as u32,
                length: 0x40 + next() % 0x100,
                kind: next() % 2,
                position: next() % 0x40,
                position_fraction: next() << 8,
                step: next() % 3,
                step_fraction: next() << 8,
                left: (next() % 0x2000) as i32 - 0x1000,
                right: (next() % 0x2000) as i32 - 0x1000,
                ..MixerChannel::default()
            })
            .collect();
        let prepare =
            |mix: &mut Vec<u32>, channels: &mut Vec<MixerChannel>, vars: &mut [u32; 2]| {
                mix.fill(0xAAAA_AAAA);
                channels.copy_from_slice(&setup);
                *vars = [shift, count];
            };
        prepare(&mut mix, &mut channels, &mut vars);
        let before = Regs {
            eax: bytes,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (mix.clone(), channels.clone());
        prepare(&mut mix, &mut channels, &mut vars);
        let done = unsafe {
            mix_channels(
                bytes,
                shift,
                mix.as_mut_ptr() as u32,
                channels.as_mut_ptr() as u32,
                count,
            )
        };
        let first = (0..mix.len()).find(|&i| mix[i] != theirs.0[i]);
        t.check(
            done.is_ok() && first.is_none() && theirs.1 == channels && ro.eax == bytes,
            || {
                format!(
                    "case {case} (seed {seed:#x}), {bytes:#x} bytes, shift {shift}, {count} channels: rust {done:?}, first dword differing {first:x?}; channels original {:x?}; rust {:x?}",
                    theirs.1, channels
                )
            },
        );
    }
    t.finish()
}
