//! `audio::write_mix` (0x40C03A): the work buffer clipped into the primary buffer's two
//! locked parts.
//!
//! In:  nothing (`g_lockedParts`, `g_soundMode`)
//! Out: eax .. ebp as its loops leave them (the one caller, `audio::mix`, keeps its own)
//!
//! See `audio/mixer.rs`; `sub_40C087`, `sub_40C0E0`, `sub_40C153` (the three formats)
//! inlined. Refuses sixteen bits with a right sample that clips (the original loops for
//! ever there: section 16).
//! Compared in shadow mode (the two parts). Self-test: a redirected copy (0x40C03A ..
//! 0x40C18C with the three) whose mode, buffer and parts are the test's, 4 formats x 2
//! splits x 4 loudnesses (the loudest not for sixteen bits).

use crate::audio::fill_buffer::LOCKED_PARTS_RVA;
use crate::audio::mixer::*;
use crate::audio::mixer_harness::dispatch_written;
use crate::audio::voices::SOUND_MODE_RVA;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::write_mix", rva(0x40C03A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub unsafe fn write_mix_in_game() -> Result<(), Refused> {
    unsafe {
        let parts = (game_addr(LOCKED_PARTS_RVA) as *const [u32; 4]).read_unaligned();
        write_mix(
            game_read(SOUND_MODE_RVA),
            game_addr(MIX_BUFFER_RVA) as u32,
            parts,
        )
    }
}

fn rust(_: &mut Regs) -> Result<(), Refused> {
    unsafe { write_mix_in_game() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let parts = unsafe { (game_addr(LOCKED_PARTS_RVA) as *const [u32; 4]).read_unaligned() };
    let mut regions = vec![(parts[0], (parts[1] as usize).min(0x10000))];
    if parts[2] != 0 {
        regions.push((parts[2], (parts[3] as usize).min(0x10000)));
    }
    dispatch_written(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mix = vec![0i32; MIX_BUFFER_LEN as usize / 4];
    let mut out = vec![0u8; 0x1000];
    let mut vars = Box::new([0u32; 5]);
    let at = vars.as_mut_ptr() as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40C03A,
            len: 0x152,
            entry: &[0xF7, 0x05, 0xBC, 0x82, 0x4F, 0x00, 0x10, 0x00, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x4F82BC, at),
            (0x952AF8, mix.as_mut_ptr() as u32),
            (0x506A98, at + 4),
            (0x506A9C, at + 8),
            (0x506AA0, at + 12),
            (0x506AA4, at + 16),
        ],
        17,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let base = out.as_mut_ptr() as u32;
    for mode in [0x110u32, 0x210, 0x220, 0x221] {
        for split in [(0x400u32, 0u32), (0x180, 0x280)] {
            for loud in [0x1000i32, 0x10_0000, 0x400_0000, 0x7FFF_FFFF] {
                // Sixteen bits are only tried where the right samples do not clip (above
                // 0x2000_0000 before the shift): the original never ends where they do.
                if mode & 0x10 == 0 && loud > 0x2000_0000 {
                    continue;
                }
                let parts = [
                    base,
                    split.0,
                    if split.1 == 0 { 0 } else { base + 0x800 },
                    split.1,
                ];
                let prepare = |mix: &mut Vec<i32>, out: &mut Vec<u8>, vars: &mut [u32; 5]| {
                    for (i, m) in mix.iter_mut().enumerate() {
                        *m = (i as i32).wrapping_mul(0x9E37_79B9u32 as i32) % loud;
                    }
                    out.fill(0xAA);
                    *vars = [mode, parts[0], parts[1], parts[2], parts[3]];
                };
                prepare(&mut mix, &mut out, &mut vars);
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = out.clone();
                prepare(&mut mix, &mut out, &mut vars);
                let done = unsafe { write_mix(mode, mix.as_ptr() as u32, parts) };
                let first = (0..out.len()).find(|&i| out[i] != theirs[i]);
                t.check(done.is_ok() && first.is_none(), || {
                    format!(
                        "mode {mode:#x}, parts {split:x?}, loudness {loud:#x}: rust {done:?}, first byte differing {first:x?}"
                    )
                });
            }
        }
    }
    t.finish()
}
