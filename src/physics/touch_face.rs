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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, EDX, ESI, Regs};
use crate::math::next_random::next_random;
use crate::physics::flow::{a, c0, g, p};
use crate::physics::place_sound_object::place_sound_object;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::drive::lit_faces::{LIST_ROOM, Touch, fall_mark, lucky, touch};

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
    let _all = [
        (a(LIT + 4 * count as usize), 4),
        (face, 0x40),
        (a(LIT_COUNT), 4),
    ];
    let eax = (|| -> Result<_, crate::errors::Refused> {
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
    })()?;
    Ok(left.with(EAX, eax))
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { touch_face() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
