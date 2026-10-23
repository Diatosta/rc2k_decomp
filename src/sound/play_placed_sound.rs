//! `sound::play_placed_sound` (0x40F11C): plays a sound that has a place around the
//! listener.
//!
//! In:  esi = the request, ten dwords: +0 the sound (bit 0x8000: none), +4 float (a
//!      pitch, by its place in the command; a reading), +8 float volume, +0xC, +0x10,
//!      +0x14 floats: where it is, +0x20 float factor for +4, +0x24 float factor for
//!      the volume
//! Out: every register preserved; the request is changed in place
//! Nothing for a sound with bit 0x8000.
//! Without bit 0x100000 in `g_soundMode` (no 3D): the volume = request's volume x the
//! sample record's float +0x24 x the request's +0x24, as a whole number held to 0 ..
//! 0x40 and stored over +8; the angle `fpatan` of +0x14 over +0xC is stored at +0x18
//! and 64.0 - |angle| x 40.74 (`flt_512500`, `flt_512504`), as a whole number, at +0x1C:
//! the place between left and right, which goes as 0 when +0xC is zero. With bit
//! 0x2000000 of the mode (Dolby) and an angle below zero its second byte is inverted.
//! With 3D: the volume as a whole number (not stored) and the three floats as they are.
//! Then +4 = +4 x +0x20 as a whole number, and `sound::play_sound` with eax the sound,
//! ebx the volume, ecx the place (3D: ecx, edi, ebp the three floats), edx = +4.
//! The x87 is used with the original's own instructions, so that the rounding is its.
//! Eight callers (the race's sounds). Shadow mode: the Rust version on a copy of the
//! request without sending, then the original, and the two requests compared.
//! Self-test: a redirected copy whose mode and sample records are the test's.
//! Refuses a null request and, without 3D, a sound past the 0x1CD sample records.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::{game_addr, game_read, rva};
use rc2k_core::sound::placed::{self, MODE_3D};

// ---- game data ----

pub const MODE_RVA: usize = rva(0x4F82BC); // g_soundMode
pub const SAMPLES_RVA: usize = rva(0x50A7A4); // unk_50A7A4
const CENTRE_RVA: usize = rva(0x512500); // flt_512500
const PER_ANGLE_RVA: usize = rva(0x512504); // flt_512504
const SAMPLE_LEN: usize = 0x40;
// ---- the x87's arithmetic, with the original's instructions ----

macro_rules! x87 {
    ($($line:literal),+; $($name:ident = $value:expr),+) => {
        unsafe {
            core::arch::asm!(
                $($line),+,
                $($name = in(reg) $value),+,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            )
        }
    };
}

/// `fld a; fmul b; fmul c; fistp`.
fn product3_to_int(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]"; v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fistp`.
pub fn product_to_int(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fistp`.
fn to_int(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", "fistp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    out
}

/// `fld y; fld x; fpatan; fst angle; fabs; fmul per_angle; fsubr centre; fistp`: the
/// angle's bits and the whole number.
fn angle_and_place(y: u32, x: u32, per_angle: u32, centre: u32) -> (u32, u32) {
    let (v, mut out) = ([y, x, per_angle, centre], [0u32; 2]);
    x87!("fld dword ptr [{v}]", "fld dword ptr [{v} + 4]", "fpatan", "fst dword ptr [{o}]",
        "fabs", "fmul dword ptr [{v} + 8]", "fsubr dword ptr [{v} + 12]",
        "fistp dword ptr [{o} + 4]"; v = v.as_ptr(), o = out.as_mut_ptr());
    (out[0], out[1])
}

// ---- Rust version ----

/// `mode` is the sound mode and `samples` the address of the sample records. Outside the
/// 3D modes, where `sound::play_sound` replaces it, the original hands on its caller's ebp.
pub unsafe fn play_placed_sound(request: u32, mode: u32, samples: usize) -> Result<(), BadInput> {
    unsafe {
        let request = request as *mut u32;
        if request.is_null() {
            return Err(BadInput("no request"));
        }
        let get = |n: usize| request.add(n).read_unaligned();
        let put = |n: usize, value: u32| request.add(n).write_unaligned(value);
        // Converted: `rc2k_core::sound::placed` (`played`, `sample_of`, `volume`,
        // `place`); the products and the angle are the original's x87.
        let sound = get(0);
        if !placed::played(sound) {
            return Ok(());
        }
        let (ebx, ecx, edi, ebp);
        if mode & MODE_3D == 0 {
            let number = placed::sample_of(sound)?;
            let record = samples + number as usize * SAMPLE_LEN;
            let factor = ((record + 0x24) as *const u32).read_unaligned();
            let volume = placed::volume(product3_to_int(get(2), factor, get(9)));
            put(2, volume);
            let (angle, place) = angle_and_place(
                get(5),
                get(3),
                game_read(PER_ANGLE_RVA),
                game_read(CENTRE_RVA),
            );
            put(6, angle);
            put(7, place);
            let place = placed::place(place, get(3), angle, mode);
            (ebx, ecx, edi, ebp) = (volume, place, record as u32, 0);
        } else {
            (ebx, ecx, edi, ebp) = (to_int(get(2)), get(3), get(4), get(5));
        }
        let edx = product_to_int(get(1), get(8));
        put(1, edx);
        crate::sound::play_sound::play_sound(sound, ebx, edx, [ecx, edi, ebp]);
        Ok(())
    }
}

/// In the game.
pub fn play_placed_sound_in_game(request: u32) -> Result<(), BadInput> {
    unsafe { play_placed_sound(request, game_read(MODE_RVA), game_addr(SAMPLES_RVA)) }
}
