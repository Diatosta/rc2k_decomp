//! `sound::surface_sound` (0x40E068): one of a car's sounds of what its wheels run on.
//!
//! In:  ebx the car's sound object, ebp the surface's sound set, esi the request
//! Out: eax the place's last dword (+0x34), ecx and edx the volume and rate
//!      (`sound::sample_settings` of the sample's own); everything else preserved
//! The set: +0 1 for a set of eight (+4 ..; the sound by the object's +0 & 3 and, 4
//! further for +0xC 1), else four (by +0 & 3). Its sound replaces the object's +0x144
//! (sets of eight) or +0x148; a changed one has the old one stopped first
//! (`sound::stop_sound`). Then the request: the sound, +4 the rate as a float, +8
//! `flt_516398` (`fld` / `fstp`), the place +0x2C .. +0x34, and
//! `sound::play_placed_sound` (a tail jump). Two callers, `sound::surface_sounds`.
//! Refuses (before anything) a null object, set or request, a set of eight with +0xC
//! over 1, and a sound past the sample records.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87::from_int;
use crate::math::x87_float::ld;
use crate::scene::x87_mem::fstp;
use crate::sound::sample_settings::{OWN, record, sample_settings};
use rc2k_core::sound::surfaces;

// ---- Rust version ----

/// Object `o`'s sound of the surface's sound `set`, through `request`.
pub fn surface_sound(o: u32, set: u32, request: u32) -> Result<(), Refused> {
    if o == 0 || set == 0 || request == 0 {
        return Err(BadInput("no object, set or request").into());
    }
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    let set_word = |at: u32| (o + at) as *mut u32;
    let set = set as *const u32;
    let q = request as *mut u32;
    // Converted: `rc2k_core::sound::surfaces::pick`.
    let eight = unsafe { set.read_unaligned() } == 1;
    let (n, slot) = surfaces::pick(eight, w(0), || w(0xC)).map_err(BadInput::from)?;
    let sound = unsafe { set.add(1 + n as usize).read_unaligned() };
    record(sound)?;
    if sound != w(slot) {
        let old = w(slot);
        unsafe {
            set_word(slot).write_unaligned(sound);
            q.write_unaligned(old);
        }
        crate::sound::turn_parts::stop(request)?;
    }
    let sound = w(slot);
    unsafe { q.write_unaligned(sound) };
    let (_, rate) = sample_settings(sound, OWN, OWN)?;
    unsafe { q.add(1).write_unaligned(from_int(rate as i32)) };
    x87_1(&crate::sound::store::SURFACE_SOUNDS, request as usize);
    for k in 0..3 {
        unsafe { q.add(3 + k).write_unaligned(w(0x2C + 4 * k as u32)) };
    }
    crate::sound::turn_parts::placed(request)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[si + 8] = [sf + 0x40]`
fn x87_1(sf: &crate::sound::store::SurfaceSounds, si: usize) {
    unsafe {
        let si = si as *mut u8;
        let t0 = ld(sf.volume.bits());
        fstp(si, 8, t0);
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(sf, si) "fld dword ptr [{sf} + 0x40]", "fstp dword ptr [{si} + 0x8]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "surface_sound line 52",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0x40, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(8, Out::F)],
                },
            ],
            ulps: 0,
            reference: |a| unsafe { ref_0(a[0], a[1]) },
            plain: |a| {
                x87_1(
                    unsafe { &*(a[0] as *const crate::sound::store::SurfaceSounds) },
                    a[1],
                )
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
