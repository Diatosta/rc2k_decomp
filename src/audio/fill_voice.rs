//! `audio::fill_voice` (0x40B06E): a new voice filled for its sample record.
//!
//! In:  eax = the voice it shares data with (0: none), ebx = the record, esi = the voice
//! Out: every register preserved
//!
//! Without a shared voice: data, length, format, frames and rate from `g_sampleFile`, no
//! source, no next copy, the record's `data_key`. With one: the same five from it, it as
//! the source, the voice added at the end of the record's chain (unless it is on it
//! already), no next copy and no `data_key`. Then from the record: flags 1 with
//! DirectSound's buffers (`g_soundMode` bit 0x10000) else 2, sound, kind, scale,
//! priority, volume, the pan's low byte (signed), pan sign 0, the frequency (or the
//! voice's own for `OWN_FREQUENCY`), the place, `field_50` from its `field_24`, the volume
//! factor (0x40, or what the record's points at); state 0, dirty bits 0xF more, no
//! channel, no list link. One caller, command 3 (`sub_40AEB9`).
//! Refuses a shared voice for a record without voices, a chain without its end.
//! Compared in shadow mode. Self-test: a redirected copy whose sample file and mode are the
//! test's, records and voices of the test's, 48 cases.

use crate::audio::samples::*;
use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::fill_voice", rva(0x40B06E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub unsafe fn fill_voice_in_game(at: u32, record: u32, shared: u32) -> Result<(), Refused> {
    unsafe {
        let file = *(game_addr(SAMPLE_FILE_RVA) as *const SampleFile);
        fill_voice(at, record, shared, &file, game_read(SOUND_MODE_RVA))
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { fill_voice_in_game(r.esi, r.ebx, r.eax) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(
        game_addr(VOICES_RVA) as u32,
        VOICES as usize * std::mem::size_of::<Voice>(),
    )];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = Box::new([0u32; 9]);
    let at = vars.as_mut_ptr() as u32;
    // vars: the sample file's eight dwords (data .. frames), then the mode.
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40B06E,
            len: 0x124,
            entry: &[0x60, 0x8B, 0xD3, 0x85, 0xC0],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x506938, at),
            (0x50693C, at + 4),
            (0x506940, at + 8),
            (0x506948, at + 16),
            (0x506954, at + 28),
            (0x4F82BC, at + 32),
        ],
        6,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let mut voices = vec![Voice::default(); 5];
    let base = voices.as_mut_ptr() as u32;
    let vat = |n: u32| base + n * std::mem::size_of::<Voice>() as u32;
    let mut factor = Box::new(0x33u32);
    let factor_at = &mut *factor as *mut u32 as u32;
    let mut case = 0u32;
    for mode in [0x1_0000u32, 0x4_0000] {
        for frequency in [22050u32, OWN_FREQUENCY] {
            for with_factor in [false, true] {
                // shared: none; voice 1 with the chain 1 -> 2; with the chain 1 -> 2 -> 4
                // and 4 the voice itself; the voice first.
                for (shared, chain, target) in [
                    (0u32, &[1u32][..], 3u32),
                    (1, &[1, 2][..], 3),
                    (2, &[1, 2, 4][..], 4),
                    (1, &[3][..], 3),
                    (0, &[][..], 3),
                    (4, &[1][..], 3),
                ] {
                    case += 1;
                    let mut record = SampleRecord {
                        number: 0x123,
                        group: 2,
                        first_voice: chain.first().map_or(0, |&n| vat(n)),
                        data_key: 0xD000 + case,
                        priority: 0xFFFF_FFFE,
                        voice_kind: 3,
                        copies: 2,
                        scale: 0x30,
                        volume_factor: if with_factor { factor_at } else { 0 },
                        field_24: 0x3F00_0000,
                        volume: 0x41,
                        pan: 0x1C0,
                        frequency,
                        place: [1, 2, 3],
                    };
                    let prepare = |voices: &mut Vec<Voice>, vars: &mut [u32; 9]| {
                        for (n, v) in voices.iter_mut().enumerate() {
                            let words = v as *mut Voice as *mut u32;
                            for i in 0..0x1C {
                                unsafe {
                                    words.add(i).write(0xE000_0000 | (n as u32) << 8 | i as u32)
                                };
                            }
                        }
                        for (i, &n) in chain.iter().enumerate() {
                            voices[n as usize].next_copy = chain.get(i + 1).map_or(0, |&m| vat(m));
                        }
                        *vars = [0x4000, 0x400, 0x221, 2, 11025, 16, 4, 0x100, mode];
                    };
                    let shared_at = if shared == 0 { 0 } else { vat(shared) };
                    prepare(&mut voices, &mut vars);
                    let before = Regs {
                        eax: shared_at,
                        ebx: &mut record as *mut SampleRecord as u32,
                        esi: vat(target),
                        ..Regs::SENTINEL
                    };
                    let mut ro = before;
                    unsafe { call_game(&mut ro, copy.addr()) };
                    let theirs = voices.clone();
                    prepare(&mut voices, &mut vars);
                    let file = unsafe { *(at as *const SampleFile) };
                    let mine =
                        unsafe { fill_voice(vat(target), before.ebx, shared_at, &file, mode) };
                    t.check(
                        mine.is_ok() && theirs == voices && ro.preserved_except(&before, 0),
                        || {
                            format!(
                                "case {case}, mode {mode:#x}, shared {shared}, chain {chain:?}: rust {mine:?}; original {theirs:x?}; rust {voices:x?}"
                            )
                        },
                    );
                }
            }
        }
    }
    t.finish()
}
