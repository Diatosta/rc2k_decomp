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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::physics::flow::{base, x87_run};
use crate::sound::store::StageSlot;
use crate::sound::store::SurfaceSlot;
use rc2k_core::sound::surfaces;

/// flt_51638C, flt_516394: the two differences whose signs pick the wheel.
const ACROSS: SurfaceSlot = SurfaceSlot::at(0x51638C);
const ALONG: SurfaceSlot = SurfaceSlot::at(0x516394);
/// dword_51655C: a surface's set, 16 of them.
const SETS_OF: StageSlot = StageSlot::at(0x51655C);
/// off_51639C: the sets (pairs of pointers, seven).
const SETS: usize = 0x51639C;
/// dword_4F66F1, dword_4F66F5: the two surfaces (unaligned).
const SURFACES: [crate::sound::store::WheelsSlot; 2] = [
    crate::sound::store::WheelsSlot::at(0x4F66F1),
    crate::sound::store::WheelsSlot::at(0x4F66F5),
];

fn g(x: impl crate::store::Ida) -> u32 {
    unsafe { crate::store::ida_read(x) }
}

// ---- Rust version ----

/// The surface of the wheel picked (of object `o`), its set and `sound::surface_sound`
/// (the other wheel of the pair, the original's edx, it does not read).
fn one(o: u32, request: u32, second: usize) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::surfaces` (`wheel`, `quieter`, `set_pair`).
    let (first, _) = surfaces::wheel(g(ACROSS), g(ALONG));
    let wheel = o + surfaces::WHEELS[first];
    let surface = unsafe { ((wheel + 0x10) as *const u32).read_unaligned() };
    if second == 0 {
        unsafe { ((o + 0x294) as *mut u32).write_unaligned(surface) };
    }
    unsafe { crate::store::game_write(SURFACES[second], surface) };
    if surfaces::quieter(surface) {
        let _b = base();
        if second == 0 {
            x87_run!(
                        "fld dword ptr [{sf} + 0x40]",
                        "mov dword ptr [{t}], 0x3E800000",
                        "fmul dword ptr [{t}]",
                        "fstp dword ptr [{sf} + 0x40]";
                t = crate::session::MACHINE.scratch.addr(), sf = crate::sound::store::surface::base()
            );
        } else {
            x87_run!(
                        "fld dword ptr [{sf} + 0x40]",
                        "mov dword ptr [{t}], 0x3DCCCCCD",
                        "fmul dword ptr [{t}]",
                        "fstp dword ptr [{sf} + 0x40]";
                t = crate::session::MACHINE.scratch.addr(), sf = crate::sound::store::surface::base()
            );
        }
    }
    let pair =
        surfaces::set_pair(surface, |n| g(SETS_OF + 4 * n as usize)).map_err(BadInput::from)?;
    let set = g(SETS + 4 * second + 8 * pair as usize);
    crate::sound::surface_sound::surface_sound(o, set, request)
}

/// The wheels' surface sounds of object `o`, through `request`.
pub fn surface_sounds(o: u32, request: u32) -> Result<(), Refused> {
    if o == 0 || request == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let (bx, _b) = (o, base());
    {
        x87_run!(
                ".byte 0xD9, 0xEE",
                "fsub dword ptr [{bx} + 0x4C]",
                "fadd dword ptr [{bx} + 0x60]",
                "fsub dword ptr [{bx} + 0x74]",
                "fadd dword ptr [{bx} + 0x88]",
                "fstp dword ptr [{sf} + 0x34]",
                ".byte 0xD9, 0xEE",
                "fadd dword ptr [{bx} + 0x4C]",
                "fadd dword ptr [{bx} + 0x60]",
                "fsub dword ptr [{bx} + 0x74]",
                "fsub dword ptr [{bx} + 0x88]",
                "fstp dword ptr [{sf} + 0x3c]",
                "fld dword ptr [{bx} + 0x44]",
                "fadd dword ptr [{bx} + 0x58]",
                "fadd dword ptr [{bx} + 0x6C]",
                "fadd dword ptr [{bx} + 0x80]",
                "fmul dword ptr [{sf} + 0xc]",
                "fistp dword ptr [{bx} + 0x284]",
                "fld dword ptr [{bx} + 0x4C]",
                "fadd dword ptr [{bx} + 0x60]",
                "fadd dword ptr [{bx} + 0x74]",
                "fadd dword ptr [{bx} + 0x88]",
                "fmul dword ptr [{sf} + 0x10]",
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
                "fmul dword ptr [{sf} + 0x4]",
                "fstp dword ptr [{sf} + 0x40]";
            bx = bx, sf = crate::sound::store::surface::base()
        );
        one(o, request, 0)?;
        x87_run!(
                ".byte 0xD9, 0xEE",
                "fsub dword ptr [{bx} + 0x50]",
                "fadd dword ptr [{bx} + 0x64]",
                "fsub dword ptr [{bx} + 0x78]",
                "fadd dword ptr [{bx} + 0x8C]",
                "fstp dword ptr [{sf} + 0x34]",
                ".byte 0xD9, 0xEE",
                "fadd dword ptr [{bx} + 0x50]",
                "fadd dword ptr [{bx} + 0x64]",
                "fsub dword ptr [{bx} + 0x78]",
                "fsub dword ptr [{bx} + 0x8C]",
                "fstp dword ptr [{sf} + 0x3c]",
                "fld dword ptr [{bx} + 0x50]",
                "fadd dword ptr [{bx} + 0x64]",
                "fadd dword ptr [{bx} + 0x78]",
                "fadd dword ptr [{bx} + 0x8C]",
                "fmul dword ptr [{sf} + 0x10]",
                "fistp dword ptr [{bx} + 0x290]",
                "fld dword ptr [{bx} + 0x50]",
                "fadd dword ptr [{bx} + 0x64]",
                "fadd dword ptr [{bx} + 0x78]",
                "fadd dword ptr [{bx} + 0x8C]",
                "fmul dword ptr [{sf} + 0x4]",
                "fstp dword ptr [{sf} + 0x40]";
            bx = bx, sf = crate::sound::store::surface::base()
        );
        one(o, request, 1)
    }
}
