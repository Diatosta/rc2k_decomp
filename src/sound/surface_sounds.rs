//! `sound::surface_sounds` (0x40DE9B): a car's two sounds of what its wheels run on.
//!
//! In:  ebx the car's sound object, esi the request
//! Out: every register preserved (`pusha` / `popa`)
//! The four wheels are records of five dwords at +0x44, +0x58, +0x6C, +0x80 (+8 and +0xC
//! floats, +0x10 the surface). From the floats at +8: `flt_51638C` (front less back,
//! by the signs), `flt_516394` (left less right), +0x284 = the sum of the four +0
//! floats x 0.4 and +0x28C = the sum at +8 x 64 (`fistp`), `flt_516398` = the products'
//! sum x 0.1: a reading. The wheel whose surface counts is the front or back pair by
//! the sign bit of the first (negative: +0x44 / +0x6C), then the second of the pair when
//! the other is negative; its surface goes to +0x294 and `dword_4F66F1`; on 7 and 8 the
//! volume (`flt_516398`) x 0.25. The surface's sound set is `off_51639C[2 x
//! dword_51655C[surface]]`, and `sound::surface_sound` (ebp the set, edx the other
//! wheel). Then the same from the floats at +0xC (+0x290 = their sum x 64, the volume
//! their sum x 0.1, the surface into `dword_4F66F5`, x 0.1 on 7 and 8) with the sets
//! of `off_5163A0`. The x87 code is the original's (through `dword_5183CC` for the
//! constants). One caller, `sound::car_sounds`.
//! Refuses a null object or request, a surface past the 16 of `dword_51655C` and a set
//! past the seven pairs: what was written is put back (a sound already sent stays).
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose two calls go to a stand-in, against the Rust version, on an
//! object of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::physics::flow::{base, guard, x87_run};
use crate::service::harness::{Region, capture, restore};
use crate::sound::turn_parts::{
    OBJECT_LEN, Part, REQUEST, both, common_regions, copy_of, dispatch_turn, f, run_copy,
    test_object, turn_regions,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::surface_sounds", rva(0x40DE9B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x1CD;
const OUTS: [usize; 2] = [0xFE, 0x1C6];
const ENTRY: [u8; 6] = [0x60, 0xD9, 0xEE, 0xD8, 0x63, 0x4C];
const SURFACE_SOUND: u32 = 0x40E068;
/// flt_51638C, flt_516394: the two differences whose signs pick the wheel.
const ACROSS: usize = 0x51638C;
const ALONG: usize = 0x516394;
/// dword_51655C: a surface's set, 16 of them.
const SETS_OF: usize = 0x51655C;
/// off_51639C: the sets (pairs of pointers, seven).
const SETS: usize = 0x51639C;
/// dword_4F66F1, dword_4F66F5: the two surfaces (unaligned).
const SURFACES: [usize; 2] = [0x4F66F1, 0x4F66F5];

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

// ---- Rust version ----

/// The wheel whose surface counts (the original's two `lea` pairs and the sign tests).
fn wheel(c: &mut Regs) {
    (c.eax, c.edx) = (c.ebx + 0x44, c.ebx + 0x6C);
    if (g(ACROSS) as i32) >= 0 {
        (c.eax, c.edx) = (c.ebx + 0x58, c.ebx + 0x80);
    }
    if (g(ALONG) as i32) < 0 {
        c.eax = c.edx;
    }
}

/// The surface of the wheel picked, its set (ebp) and `sound::surface_sound`.
fn one(c: &mut Regs, second: usize, part: Part) -> Result<(), Refused> {
    wheel(c);
    c.eax = unsafe { ((c.eax + 0x10) as *const u32).read_unaligned() };
    if second == 0 {
        unsafe { ((c.ebx + 0x294) as *mut u32).write_unaligned(c.eax) };
    }
    unsafe { game_write(rva(SURFACES[second]), c.eax) };
    if c.eax == 7 || c.eax == 8 {
        let b = base();
        if second == 0 {
            x87_run!(
                "fld dword ptr [{b} + 0x116398]",
                "mov dword ptr [{b} + 0x1183CC], 0x3E800000",
                "fmul dword ptr [{b} + 0x1183CC]",
                "fstp dword ptr [{b} + 0x116398]";
                b = b
            );
        } else {
            x87_run!(
                "fld dword ptr [{b} + 0x116398]",
                "mov dword ptr [{b} + 0x1183CC], 0x3DCCCCCD",
                "fmul dword ptr [{b} + 0x1183CC]",
                "fstp dword ptr [{b} + 0x116398]";
                b = b
            );
        }
    }
    if c.eax >= 16 {
        return Err(BadInput("a surface past the 16 of dword_51655C").into());
    }
    c.eax = g(SETS_OF + 4 * c.eax as usize);
    if c.eax >= 7 {
        return Err(BadInput("a surface set past the seven pairs").into());
    }
    c.ebp = g(SETS + 4 * second + 8 * c.eax as usize);
    part(SURFACE_SOUND, c)
}

pub fn surface_sounds_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if r.ebx == 0 || r.esi == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let mut c = *r;
    let (bx, b) = (c.ebx, base());
    guard(&turn_regions_with(bx), || {
        x87_run!(
            ".byte 0xD9, 0xEE",
            "fsub dword ptr [{bx} + 0x4C]",
            "fadd dword ptr [{bx} + 0x60]",
            "fsub dword ptr [{bx} + 0x74]",
            "fadd dword ptr [{bx} + 0x88]",
            "fstp dword ptr [{b} + 0x11638C]",
            ".byte 0xD9, 0xEE",
            "fadd dword ptr [{bx} + 0x4C]",
            "fadd dword ptr [{bx} + 0x60]",
            "fsub dword ptr [{bx} + 0x74]",
            "fsub dword ptr [{bx} + 0x88]",
            "fstp dword ptr [{b} + 0x116394]",
            "fld dword ptr [{bx} + 0x44]",
            "fadd dword ptr [{bx} + 0x58]",
            "fadd dword ptr [{bx} + 0x6C]",
            "fadd dword ptr [{bx} + 0x80]",
            "fmul dword ptr [{b} + 0x116364]",
            "fistp dword ptr [{bx} + 0x284]",
            "fld dword ptr [{bx} + 0x4C]",
            "fadd dword ptr [{bx} + 0x60]",
            "fadd dword ptr [{bx} + 0x74]",
            "fadd dword ptr [{bx} + 0x88]",
            "fmul dword ptr [{b} + 0x116368]",
            "fistp dword ptr [{bx} + 0x28C]",
            ".byte 0xD9, 0xEE",
            "fld dword ptr [{bx} + 0x44]",
            "fmul dword ptr [{bx} + 0x4C]",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{bx} + 0x58]",
            "fmul dword ptr [{bx} + 0x60]",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{bx} + 0x6C]",
            "fmul dword ptr [{bx} + 0x74]",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{bx} + 0x80]",
            "fmul dword ptr [{bx} + 0x88]",
            ".byte 0xDE, 0xC1",
            "fmul dword ptr [{b} + 0x11635C]",
            "fstp dword ptr [{b} + 0x116398]";
            bx = bx, b = b
        );
        one(&mut c, 0, part)?;
        x87_run!(
            ".byte 0xD9, 0xEE",
            "fsub dword ptr [{bx} + 0x50]",
            "fadd dword ptr [{bx} + 0x64]",
            "fsub dword ptr [{bx} + 0x78]",
            "fadd dword ptr [{bx} + 0x8C]",
            "fstp dword ptr [{b} + 0x11638C]",
            ".byte 0xD9, 0xEE",
            "fadd dword ptr [{bx} + 0x50]",
            "fadd dword ptr [{bx} + 0x64]",
            "fsub dword ptr [{bx} + 0x78]",
            "fsub dword ptr [{bx} + 0x8C]",
            "fstp dword ptr [{b} + 0x116394]",
            "fld dword ptr [{bx} + 0x50]",
            "fadd dword ptr [{bx} + 0x64]",
            "fadd dword ptr [{bx} + 0x78]",
            "fadd dword ptr [{bx} + 0x8C]",
            "fmul dword ptr [{b} + 0x116368]",
            "fistp dword ptr [{bx} + 0x290]",
            "fld dword ptr [{bx} + 0x50]",
            "fadd dword ptr [{bx} + 0x64]",
            "fadd dword ptr [{bx} + 0x78]",
            "fadd dword ptr [{bx} + 0x8C]",
            "fmul dword ptr [{b} + 0x11635C]",
            "fstp dword ptr [{b} + 0x116398]";
            bx = bx, b = b
        );
        one(&mut c, 1, part)
    })
}

/// `turn_regions` and the object (which may not be one of the eight).
fn turn_regions_with(o: u32) -> Vec<Region> {
    let mut all = turn_regions();
    all.push((o, OBJECT_LEN));
    all
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, surface_sounds_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40DE9B, LEN, &ENTRY, &OUTS, &[SURFACE_SOUND]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let mut object = test_object(0x99, &[]);
    let o = object.as_mut_ptr() as u32;
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut all: Vec<Region> = vec![(o, OBJECT_LEN), (a(ACROSS), 0x10), (a(SURFACES[0]), 8)];
    all.extend(common_regions());
    let saved = capture(&all);
    let signs: [[f32; 4]; 5] = [
        [1.0, 1.0, 1.0, 1.0],
        [-1.0, 2.0, 0.5, -3.0],
        [3.0, -1.0, -2.0, 0.25],
        [-0.0, 0.0, -0.0, 0.0],
        [-5.0, -5.0, 4.0, 4.0],
    ];
    let surfaces: [[u32; 4]; 4] = [[0, 1, 2, 3], [7, 8, 7, 8], [15, 4, 12, 9], [16, 16, 16, 16]];
    for (k, s) in signs.iter().enumerate() {
        for (m, surface) in surfaces.iter().enumerate() {
            let fill = || unsafe {
                let p = o as *mut u32;
                for w in 0..4usize {
                    let at = (0x44 + 0x14 * w) / 4;
                    p.add(at).write(f(0.5 + w as f32));
                    p.add(at + 2).write(f(s[w]));
                    p.add(at + 3).write(f(-s[(w + 1) % 4] * 1.5));
                    p.add(at + 4).write(surface[w]);
                }
            };
            both(
                &mut t,
                &format!("signs {k}, surfaces {m}"),
                &run,
                Regs {
                    ebx: o,
                    esi: game_addr(rva(REQUEST)) as u32,
                    ..Regs::SENTINEL
                },
                &fill,
                &all,
                surface_sounds_in,
                m == 3,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
