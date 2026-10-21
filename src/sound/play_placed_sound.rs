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

use crate::errors::BadInput;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::sound::play_sound::play_sound_in_game;
use crate::sound::queue::sent;
use rc2k_core::sound::placed::{self, MODE_3D, SAMPLES};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::play_placed_sound", rva(0x40F11C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const MODE_RVA: usize = rva(0x4F82BC); // g_soundMode
pub const SAMPLES_RVA: usize = rva(0x50A7A4); // unk_50A7A4
const CENTRE_RVA: usize = rva(0x512500); // flt_512500
const PER_ANGLE_RVA: usize = rva(0x512504); // flt_512504
const SAMPLE_LEN: usize = 0x40;
/// The request's length in dwords.
const REQUEST: usize = 10;

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

/// The request is at `entry.esi`; `mode` is the sound mode and `samples` the address of
/// the sample records. `play` gets the registers `sound::play_sound` is called with.
pub unsafe fn play_placed_sound(
    entry: &Regs,
    mode: u32,
    samples: usize,
    play: &mut dyn FnMut(&Regs),
) -> Result<(), BadInput> {
    unsafe {
        let request = entry.esi as *mut u32;
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
            (ebx, ecx, edi, ebp) = (volume, place, record as u32, entry.ebp);
        } else {
            (ebx, ecx, edi, ebp) = (to_int(get(2)), get(3), get(4), get(5));
        }
        let edx = product_to_int(get(1), get(8));
        put(1, edx);
        play(&Regs {
            eax: sound,
            ebx,
            ecx,
            edx,
            edi,
            ebp,
            ..*entry
        });
        Ok(())
    }
}

// ---- hook ----

fn request_at(at: u32) -> [u32; REQUEST] {
    std::array::from_fn(|n| unsafe { (at as *const u32).add(n).read_unaligned() })
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (mode, samples) = unsafe { (game_read(MODE_RVA), game_addr(SAMPLES_RVA)) };
    if !shadow() {
        let entry = *r;
        let done = unsafe {
            play_placed_sound(&entry, mode, samples, &mut |with| play_sound_in_game(with))
        };
        if let Err(e) = done {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    // Shadow mode: the Rust version works on a copy of the request and sends nothing.
    let mine = (r.esi != 0).then(|| {
        let before = request_at(r.esi);
        let mut copy = before;
        let entry = Regs {
            esi: copy.as_mut_ptr() as u32,
            ..*r
        };
        let done = unsafe { play_placed_sound(&entry, mode, samples, &mut |_| {}) };
        (before, copy, done.is_ok())
    });
    unsafe { call_original(r) };
    if let Some((before, mine, true)) = mine {
        let theirs = request_at(r.esi);
        if theirs != mine {
            HOOK.mismatch(|| {
                format!(
                    "mode {mode:#x}, request {before:x?}: original left {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
}

// ---- self-test ----

const LEN: usize = 0xB5;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8B, 0x06, 0xA9, 0x00, 0x80, 0x00, 0x00];
/// `sound::play_sound`.
const CALLS: [usize; 1] = [0xAE];
/// How often the code names the mode and the sample records.
const NAMED: [usize; 2] = [2, 1];

static PLAYED: Mutex<Vec<[u32; 7]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    PLAYED.lock().unwrap().push(sent(r));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x40F11C)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            Some(stand_in(0))
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    // The test's mode and sample records; the records' factors differ by record.
    let mut mode_var = Box::new(0u32);
    let mut samples = vec![0u8; SAMPLES as usize * SAMPLE_LEN];
    for n in 0..SAMPLES as usize {
        let factor = [1.0f32, 0.5, 0.25, 2.0][n % 4].to_bits();
        samples[n * SAMPLE_LEN + 0x24..][..4].copy_from_slice(&factor.to_le_bytes());
    }
    let addresses = [&raw mut *mode_var as u32, samples.as_ptr() as u32];
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let real = [MODE_RVA, SAMPLES_RVA].map(|at| game_addr(at) as u32);
        let mut patches = Vec::new();
        let mut seen = [0usize; 2];
        let mut at = 0;
        while at + 4 <= LEN {
            let named = u32::from_le_bytes(code[at..at + 4].try_into().unwrap());
            match real.iter().position(|&address| address == named) {
                Some(n) => {
                    seen[n] += 1;
                    patches.push((at, addresses[n]));
                    at += 4;
                }
                None => at += 1,
            }
        }
        if seen != NAMED {
            return t.skip("the function does not name its variables as the test expects");
        }
        for (at, value) in patches {
            copy.patch_u32(at, value);
        }
    }
    let f = |value: f32| value.to_bits();
    // (sound, +4, volume, three floats of the place, factor for +4, factor for the volume)
    let requests: [[u32; 8]; 14] = [
        [5, f(1.0), f(40.0), f(3.0), f(0.0), f(4.0), f(100.0), f(1.0)],
        [
            6,
            f(0.9),
            f(64.0),
            f(-3.0),
            f(1.0),
            f(4.0),
            f(110.5),
            f(0.5),
        ],
        [7, f(1.5), f(10.5), f(3.0), f(2.0), f(-4.0), f(64.0), f(1.0)],
        [
            8,
            f(1.0),
            f(100.0),
            f(-0.001),
            f(0.0),
            f(-50.0),
            f(100.0),
            f(1.0),
        ],
        [
            0x1CC,
            f(1.0),
            f(30.0),
            f(0.0),
            f(0.0),
            f(5.0),
            f(100.0),
            f(1.0),
        ],
        [
            9,
            f(1.0),
            f(30.0),
            f(-0.0),
            f(0.0),
            f(-5.0),
            f(100.0),
            f(1.0),
        ],
        [
            10,
            f(1.0),
            f(-20.0),
            f(7.0),
            f(0.0),
            f(0.0),
            f(100.0),
            f(1.0),
        ],
        [11, f(2.5), f(33.5), f(0.0), f(0.0), f(0.0), f(0.5), f(1.0)],
        [
            0x8005,
            f(1.0),
            f(40.0),
            f(3.0),
            f(0.0),
            f(4.0),
            f(100.0),
            f(1.0),
        ],
        [
            0xFFFF_0003,
            f(1.0),
            f(40.0),
            f(1.0e6),
            f(0.0),
            f(1.0),
            f(100.0),
            f(3.0),
        ],
        [
            12,
            f(1.0e12),
            f(1.0e12),
            f(1.0),
            f(5.0),
            f(1.0),
            f(1.0e12),
            f(1.0),
        ],
        [
            13,
            f(1.0),
            f(40.0),
            f(f32::NAN),
            f(0.0),
            f(4.0),
            f(100.0),
            f(1.0),
        ],
        [
            14,
            f(0.5),
            f(0.5),
            f(-1.0),
            f(-1.0),
            f(-1.0),
            f(1.0),
            f(1.0),
        ],
        [
            15,
            f(1.0),
            f(63.5),
            f(1.0),
            f(0.0),
            f(1000.0),
            f(99.5),
            f(1.0),
        ],
    ];
    let mut request = [0u32; REQUEST];
    for mode in [0x11221u32, 0x0244_1220, 0x0010_1220, 0x0210_0000, 0] {
        for values in &requests {
            let [sound, pitch, volume, x, y, z, pitch_factor, volume_factor] = *values;
            let filled = [
                sound,
                pitch,
                volume,
                x,
                y,
                z,
                0xAAAA_AAAA,
                0xAAAA_AAAA,
                pitch_factor,
                volume_factor,
            ];
            let before = Regs {
                esi: request.as_mut_ptr() as u32,
                ..Regs::SENTINEL
            };
            let mut run = |rust: bool| unsafe {
                *mode_var = mode;
                request = filled;
                PLAYED.lock().unwrap().clear();
                set_handler(handler);
                let ended = if rust {
                    match play_placed_sound(&before, mode, samples.as_ptr() as usize, &mut |with| {
                        PLAYED.lock().unwrap().push(sent(with))
                    }) {
                        Ok(()) => 0,
                        Err(_) => 2,
                    }
                } else {
                    let mut ro = before;
                    call_game(&mut ro, copy.addr());
                    !ro.preserved_except(&before, 0) as u32
                };
                (ended, request, PLAYED.lock().unwrap().clone())
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.0 == 0, || {
                format!("mode {mode:#x}, request {filled:x?}: original {theirs:x?}; rust {mine:x?}")
            });
        }
    }
    t.finish()
}
