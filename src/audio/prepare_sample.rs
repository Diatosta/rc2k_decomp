//! `audio::prepare_sample` (0x40B3F5): a voice's sample data prepared in place.
//!
//! In:  eax = the mode (`audio::sample_mode`), esi = the voice
//! Out: every register preserved
//!
//! Only for a voice with data, a length and a `data_key` (one that read its own file).
//! With a scale other than 0x40: every sample times the scale over 64 (`sar`), held to its
//! range (8 bits unsigned, 16 signed). With halvings in the mode (bits 0 and 1): while the
//! voice's own frequency is 200 or more and its length 8 or more, the data halved by
//! averaging pairs of frames (8 bits `shr`, 16 bits `sar`), the format's shift + 1, length
//! and frames halved, until the shift's four bits are the mode's. With bit 0x10 (eight
//! bits) and 16-bit data of 8 bytes or more: each sample's high byte + 0x80, the format
//! made eight bits, the length halved. One caller, command 3 (`sub_40AEB9`).
//! Refuses a count of 0 for any of its loops (the original's `dec ecx; jnz` would run
//! 2^32 times).
//! Compared in shadow mode (the voice and its data). Self-test: the original on samples of
//! the test's, 4 formats x 5 scales x 8 modes x 3 lengths.

use crate::audio::samples::*;
use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::prepare_sample", rva(0x40B3F5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { prepare_sample(voice(r.esi), r.eax) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = unsafe { *voice(r.esi) };
    let mut regions = vec![(r.esi, std::mem::size_of::<Voice>())];
    if v.data != 0 {
        regions.push((v.data, (v.length as usize).min(0x1000_0000)));
    }
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut data = vec![0u8; 0x400];
    let mut v = Box::new(Voice::default());
    let v_at = &mut *v as *mut Voice as u32;
    for format in [0x110u32, 0x210, 0x120, 0x220] {
        for scale in [0x40u32, 0x20, 0x80, 0, 0x7F] {
            for mode in [0u32, 1, 2, 3, 0x10, 0x11, 0x13, 0x12] {
                for (length, frequency) in [(0x400u32, 22050u32), (0x10, 11025), (0x100, 150)] {
                    let frame = (if format & 0x10 != 0 { 1 } else { 2 })
                        * (if format & 0x100 != 0 { 1 } else { 2 });
                    let prepare = |data: &mut Vec<u8>, v: &mut Voice| {
                        for (i, b) in data.iter_mut().enumerate() {
                            *b = (i as u32).wrapping_mul(0x9D) as u8 ^ (i >> 3) as u8;
                        }
                        *v = Voice {
                            data: data.as_ptr() as u32,
                            length,
                            format,
                            frames: length / frame,
                            own_frequency: frequency,
                            scale,
                            data_key: 1,
                            ..Voice::default()
                        };
                    };
                    prepare(&mut data, &mut v);
                    let before = Regs {
                        eax: mode,
                        esi: v_at,
                        ..Regs::SENTINEL
                    };
                    let mut ro = before;
                    unsafe { call_original(&mut ro) };
                    let theirs = (data.clone(), *v);
                    prepare(&mut data, &mut v);
                    let mine = unsafe { prepare_sample(&mut v, mode) };
                    let first = (0..data.len()).find(|&i| data[i] != theirs.0[i]);
                    t.check(
                        mine.is_ok() && first.is_none() && theirs.1 == *v
                            && ro.preserved_except(&before, 0),
                        || {
                            format!(
                                "format {format:#x}, scale {scale:#x}, mode {mode:#x}, length {length:#x}: rust {mine:?}, first byte differing {first:x?}; original {:x?}; rust {:x?}",
                                theirs.1, *v
                            )
                        },
                    );
                }
            }
        }
    }
    t.finish()
}
