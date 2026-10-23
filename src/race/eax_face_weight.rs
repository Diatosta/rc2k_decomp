//! `race::eax_face_weight` (0x45C881): a course face's weight in the EAX sound environment
//! (`g_eaxEnvironment`, a float sum; `dword_6D1AF0` counts the faces), by its distance from
//! car 0 in x and z (see `race/frame_draw.rs`).
//!
//! In:  ebx = the face's number
//! Out: esi the face; eax, ebx, ecx as its path leaves them (its caller `sub_45C84C` calls
//!      it inside `pusha`)
//!
//! Kind 0xFF with an object below 0x38: its +0x18 byte over the distance to its first
//! point; kinds below 0x10 with |+0x30| below 0.25: 3.0 over the distance to the middle of
//! its second and third points; each held to 16.0. The distance is the game's table root
//! (`g_rootTable` by the square sum's mantissa, `g_exponentRootTable` by its exponent,
//! x 1/65536). All x87 as the original. Shadow mode: by state (the sum, the count and the
//! dwords it passes through). Self-test: test faces and points (`g_courseFaces`,
//! `g_scenePoints` pointed at them), car 0's place set, each kind and test; all put back
//! (skipped while the root tables are empty).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, restore, state_entry};

pub fn weight_regions() -> Vec<Region> {
    [
        0x6D1DE4usize,
        0x6D1DEC,
        0x6D1DF0,
        0x6D1DD8,
        0x6D1DE0,
        0x62400C,
        0x6D1DD4,
        0x6D1AF0,
    ]
    .iter()
    .map(|&a| at(rva(a)))
    .collect()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { eax_face_weight(r.ebx, r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// A test stage for the EAX weights: faces and points of the test's, car 0 placed.
pub struct WeightStage {
    pub faces: Vec<u8>,
    pub points: Vec<u32>,
    pub kept: Vec<Region>,
    pub saved: Vec<u8>,
}

impl WeightStage {
    pub fn new(extra: &[Region]) -> Option<Self> {
        if unsafe { game_read::<u32>(rva(0x5C70F8 + 4 * 0x80)) } == 0 {
            return None;
        }
        let mut kept = weight_regions();
        kept.extend([
            at(rva(0x5CCCBC)),
            at(rva(0x5CCCC0)),
            at(rva(0x70F520)),
            at(rva(0x70F528)),
        ]);
        kept.extend_from_slice(extra);
        let saved = capture(&kept);
        let mut s = Self {
            faces: vec![0u8; 0x40 * 12],
            points: vec![0u32; 3 * 16],
            kept,
            saved,
        };
        for p in 0..16u32 {
            let i = 3 * p as usize;
            s.points[i] = p.wrapping_mul(0x9E37) % 0x40000;
            s.points[i + 1] = p * 3;
            s.points[i + 2] = p.wrapping_mul(0x7F4A) % 0x40000;
        }
        let base = s.faces.as_mut_ptr() as u32;
        for f in 0..12u32 {
            let at = base + 0x40 * f;
            let kind = [0xFFu8, 0xFF, 0x05, 0x0F, 0x12, 0xFF][f as usize % 6];
            let object = [0x10u8, 0x40, 0, 0, 0, 0x37][f as usize % 6];
            let tilt = [0.1f32, 0.3, -0.2, 0.24, 0.0, 0.5][f as usize % 6];
            unsafe {
                (at as *mut u8).write(kind);
                ((at + 1) as *mut u8).write(object);
                for k in 0..3u32 {
                    ((at + 4 + 4 * k) as *mut u32).write_unaligned((f + k * 5) % 16);
                }
                ((at + 0x18) as *mut u8).write((f * 37) as u8);
                ((at + 0x30) as *mut u32).write_unaligned(tilt.to_bits());
            }
        }
        Some(s)
    }

    /// Everything put back, then the test's faces, points and car 0's place in place.
    pub fn prepare(&self, car: (u32, u32)) {
        restore(&self.kept, &self.saved);
        unsafe {
            game_write(rva(0x5CCCBC), self.faces.as_ptr() as u32);
            game_write(rva(0x5CCCC0), self.points.as_ptr() as u32);
            game_write(rva(0x70F520), car.0);
            game_write(rva(0x70F528), car.1);
        }
    }

    pub fn end(self) {
        restore(&self.kept, &self.saved);
    }
}
