//! `physics::touch_face` (0x45E9E0): the course face touched (`dword_6D3218`, in
//! `g_courseFaces`, 0x40 bytes each) marked and listed. A face whose byte +0x11 is not 0,
//! or a full list (`g_litFaceCount` at 0x100), is left alone. Else its number goes into
//! `g_litFaces[count]` and by the contact kind `dword_6D3624`:
//! - 9 and 1, unless `unk_71A19C` is below `flt_5C5D50` (or unordered; then nothing more),
//!   and 2, 3, 8: bytes +9 = 0, +0x11 = 1, +0x13 = 0, +0x12 = `dword_6D3B70`'s low byte;
//! - 4: +0x11 = `dword_6D3B74` >> 14 held to 0x40, +0x13 = 0, +0x12 as above;
//! - 0xFF: one in about 85 (`math::next_random` & 0xFF below 3) +0x11 = 0xF, counted, no
//!   sound; else nothing more;
//! - the rest (10, 5, 6, 7, ...): nothing to the face.
//!
//! Then `physics::place_sound_object` and the count + 1.
//!
//! In:  nothing
//! Out: ebx the face's number, esi its address; when the face was free, edx the count
//!      before; eax as the kind's path leaves it (the kind, the random number, or
//!      `dword_6D3B70`); the rest kept; no flags read (callers return)
//!
//! Callers: `sub_45E3BF` (six).
//! Refuses (before anything) a count past 0x100 (the list's room); what
//! `place_sound_object` refuses (its writes put back).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with test faces (`g_courseFaces` pointed at them for the test), every kind, the
//! list empty, part full and full, the current car not listed (no sound).

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBX, EDX, ESI, RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva,
};
use crate::math::next_random::{STATE_RVA, next_random};
use crate::physics::flow::{a, c0, g, guard, p};
use crate::physics::place_sound_object::place_sound_object;
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use rc2k_core::drive::lit_faces::{LIST_ROOM, Touch, fall_mark, lucky, touch};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::touch_face", rva(0x45E9E0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `g_courseFaces`: the pointer to the course's faces.
pub const FACES: usize = 0x5CCCBC;
const FACE: usize = 0x6D3218; // dword_6D3218
const LIT: usize = 0x6D3220; // g_litFaces (0x100 dwords)
const LIT_COUNT: usize = 0x6D3620; // g_litFaceCount
const KIND: usize = crate::physics::point_hits_car::KIND;

fn put(at: u32, v: u32) {
    unsafe { (at as *mut u8).write(v as u8) };
}

/// The usual marks: +9 = 0, +0x11 = `mark`, +0x13 = 0, +0x12 `dword_6D3B70`'s low byte.
fn mark(face: u32, mark: u32, clear_9: bool) -> u32 {
    if clear_9 {
        put(face + 9, 0);
    }
    put(face + 0x11, mark);
    put(face + 0x13, 0);
    let low = g(0x6D3B70);
    put(face + 0x12, low);
    low
}

/// The Rust version.
// Converted: `rc2k_core::drive::lit_faces::{touch, fall_mark, lucky}`.
pub unsafe fn touch_face() -> Result<Left, Refused> {
    let number = g(FACE);
    let face = (number << 6).wrapping_add(g(FACES));
    let left = Left::nothing().with(EBX, number).with(ESI, face);
    if unsafe { (face as *const u8).add(0x11).read() } != 0 {
        return Ok(left);
    }
    let count = g(LIT_COUNT);
    let left = left.with(EDX, count);
    if count == LIST_ROOM {
        return Ok(left);
    }
    if count > LIST_ROOM {
        return Err(BadInput("g_litFaceCount past 0x100 (the list's room)").into());
    }
    let all = [
        (a(LIT + 4 * count as usize), 4),
        (face, 0x40),
        (a(LIT_COUNT), 4),
    ];
    let eax = guard(&all, || {
        p(LIT + 4 * count as usize, number);
        let kind = g(KIND);
        let eax = match touch(kind) {
            Touch::MarkIfFast => {
                if c0(g(0x71A19C), g(0x5C5D50)) {
                    return Ok(kind);
                }
                mark(face, 1, true)
            }
            Touch::Mark => mark(face, 1, true),
            Touch::Fall => mark(face, fall_mark(g(0x6D3B74)), false),
            Touch::Chance => {
                let (r, lucky) = lucky(unsafe { next_random() });
                if lucky {
                    put(face + 0x11, 0xF);
                    p(LIT_COUNT, count + 1);
                }
                return Ok(r);
            }
            Touch::Sound => kind,
        };
        unsafe { place_sound_object() }?;
        p(LIT_COUNT, count + 1);
        Ok(eax)
    })?;
    Ok(left.with(EAX, eax))
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { touch_face() }
}

pub fn regions() -> Vec<Region> {
    let mut all = vec![(a(LIT), 0x404), (game_addr(STATE_RVA) as u32, 4)];
    let face = (g(FACE) << 6).wrapping_add(g(FACES));
    if g(FACES) != 0 {
        all.push((face, 0x40));
    }
    // Only a listed car's object is written (`place_sound_object`); an unlisted car's
    // pointer may be stale (see `crash_contact::sound_regions`).
    let listed = crate::physics::car_sounds::listed(g(0x71B3F8)) == Ok(true);
    if listed && let Ok(o) = crate::physics::car_sounds::object() {
        all.extend([(o + 0x10, 4), (o + 0x238, 0x28)]);
    }
    all.push((a(0x6D1AEC), 4));
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let car = g(crate::physics::car_sounds::CURRENT) & 7;
    let listed = crate::physics::car_sounds::LISTED + 4 * car as usize;
    let mut all: Vec<Region> = vec![
        (a(LIT), 0x404),
        (game_addr(STATE_RVA) as u32, 4),
        (a(FACE), 4),
        (a(FACES), 4),
        (a(KIND), 4),
        (a(0x71A19C), 4),
        (a(0x6D3B70), 8),
        (a(crate::physics::car_sounds::CURRENT), 4),
        (a(listed), 4),
    ];
    let saved = capture(&all);
    let mut faces = [0u32; 0x10 * 8];
    let fs = faces.as_mut_ptr() as u32;
    all.push((fs, 0x200));
    let kinds = [9u32, 1, 2, 3, 8, 4, 0xFF, 10, 5, 6, 7, 0];
    for seed in 0..96u32 {
        let set = || {
            p(crate::physics::car_sounds::CURRENT, car);
            p(listed, u32::MAX);
            p(FACES, fs);
            let mut f = [0u32; 0x84];
            test_floats(seed * 3 + 1, &mut f);
            for (k, v) in f.iter().enumerate().take(0x80) {
                unsafe { ((fs + 4 * k as u32) as *mut u32).write(*v) };
            }
            let n = seed % 8;
            p(FACE, n);
            // Free most of the time.
            if seed % 5 != 4 {
                put(fs + 0x40 * n + 0x11, 0);
            }
            p(LIT_COUNT, [0u32, 7, 0xFF, 0x100, 0x30][(seed % 5) as usize]);
            p(KIND, kinds[(seed % 12) as usize]);
            p(0x71A19C, (f32::from_bits(f[0x80]) % 4.0).to_bits());
            p(0x6D3B70, f[0x81]);
            p(0x6D3B74, f[0x82] >> (seed % 4 * 4));
            unsafe { game_write(STATE_RVA, f[0x83]) };
        };
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            &rust,
        );
    }
    all.pop();
    restore(&all, &saved);
    t.finish()
}
