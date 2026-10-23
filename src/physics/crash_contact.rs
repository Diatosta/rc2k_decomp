//! `physics::crash_contact` (0x45EDB5): the body's deepest point against the ground.
//! `unk_71AA0C` = 0; `physics::lowest_body_point`; unless the depth (`dword_6D3068`) is
//! below `dword_613EF8` and, with `unk_71A22C` at 0, below 0 (each by the sign of the
//! `cmp`), only `physics::sound_cue_e5`. Else, for the point n (`dword_6D306C`): its three
//! floats from `dword_6D1E14[n]`, `dword_6D2114[n]`, `dword_6D2014[n]` into `unk_71AA00`
//! .., `unk_71AA0C` = 1; the contact taken (`take_contact`: the car's speed, the face's
//! normal, the turn speed, the point); the car's up (0, 1, 0 turned by `g_crashMatrix`,
//! `scene::rotate_point`) into `flt_6D31C4` .., minus its dot with the normal into
//! `flt_6D31D0`, and from that the slide limit `flt_6D3100`; `physics::contact_force`
//! into force record 21 (`g_forceRecords` +0x2A0), `physics::place_sound_e5`; the force
//! into `dword_6D3018` .., the record's point 0; `flt_6D3014` = (`unk_71A0A0` +
//! `flt_5C5D60`) / `flt_5C5D60` held at most 1, times `unk_71A160`; `unk_71AA10` =
//! `dword_6D310C`.
//!
//! In:  nothing
//! Out: the registers as `lowest_body_point` and, on the long path, `contact_force` leave
//!      them, eax the depth on the short; no flags read (the caller calls on)
//!
//! Caller: `sub_43B635`, every physics step.
//! Refuses a point past 0x3F on the long path (the tables' room), and what its calls
//! refuse; what was written is put back.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with test faces (`g_courseFaces` pointed at them for the test): depths, the
//! contact's figures and the matrices by lot, both paths, the current car not listed
//! (no sound), under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::math::x87::{loaded, mul};
use crate::math::x87_run::neg_rounded;
use crate::physics::contact_force::{RECORD, contact_force};
use crate::physics::dot_product::dot_product;
use crate::physics::flow::{a, base, copy3, fill3, g, hold_high, p, x87_run};
use crate::physics::lowest_body_point::lowest_body_point;
use crate::physics::place_sound_e5::place_sound_e5;
use crate::physics::sound_cue_e5::sound_cue_e5;
use crate::physics::touch_face::FACES;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use rc2k_core::physics::contacts;

/// Force record 21 (`g_forceRecords` + 0x2A0).
const FORCE: usize = 0x614A10;

/// The sign of a `cmp`'s difference (`js`).
pub fn sign(v: u32) -> bool {
    (v as i32) < 0
}

/// The contact of point `n` taken (0x45EE25 / 0x45F02D): the car's speed (`unk_71A170`
/// ..) into `dword_6D30A8` ..; the normal of its face (`unk_71ADC8[n]` in
/// `g_courseFaces`: +0x18, +0x30, +0x38, or with `dword_6D2914[n]` at 1 +0x1C, +0x34,
/// +0x3C) into `qword_6D3078` ..; the turn speed (`unk_71A15C` ..) into `dword_6D30B4`
/// ..; the point (`flt_6D2214[n]`, `flt_6D2314[n]`, `flt_6D2414[n]`) into
/// `dword_6D3090` ...
pub fn take_contact(n: usize) {
    copy3(0x6D30A8, 0x71A170);
    let face = (g(0x71ADC8 + 4 * n) << 6).wrapping_add(g(FACES));
    let parts = contacts::normal_offsets(g(0x6D2914 + 4 * n));
    for (k, o) in parts.iter().enumerate() {
        let v = unsafe { ((face + o) as *const u32).read_unaligned() };
        p(0x6D3078 + 4 * k, loaded(v));
    }
    copy3(0x6D30B4, 0x71A15C);
    p(0x6D3090, g(0x6D2214 + 4 * n));
    p(0x6D3094, g(0x6D2314 + 4 * n));
    p(0x6D3098, g(0x6D2414 + 4 * n));
}

/// The deepest point's number (`dword_6D306C`) for the long path; refused past 0x3F.
pub fn point_number() -> Result<usize, Refused> {
    let n = g(0x6D306C);
    contacts::check_body_point(n)?;
    Ok(n as usize)
}

// Converted: `rc2k_core::physics::contacts::{touches, normal_offsets, check_body_point}`
// (the x87 steps stay).
fn steps(r: &mut Regs) -> Result<(), Refused> {
    p(0x71AA0C, 0);
    unsafe { lowest_body_point(r) }?;
    let depth = g(0x6D3068);
    r.eax = depth;
    if !contacts::touches(depth, g(0x613EF8), g(0x71A22C) != 0) {
        unsafe { sound_cue_e5() }?;
        return Ok(());
    }
    let n = point_number()?;
    r.ebx = n as u32;
    p(0x71AA00, g(0x6D1E14 + 4 * n));
    p(0x71AA04, g(0x6D2114 + 4 * n));
    p(0x71AA08, g(0x6D2014 + 4 * n));
    p(0x71AA0C, 1);
    take_contact(n);
    let up = unsafe {
        rotate_point(
            a(0x71A08C),
            [extended(0), extended(0x3F80_0000), extended(0)],
        )
    };
    for (k, e) in up.iter().enumerate() {
        p(0x6D31C4 + 4 * k, rounded(*e));
    }
    p(
        0x6D31D0,
        neg_rounded(unsafe { dot_product(a(0x6D31C4), a(0x6D3078)) }),
    );
    x87_run!(
        "fld dword ptr [{b} + 0x2D31D0]",
        "fadd dword ptr [{b} + 0x1C5D38]",
        "fdiv dword ptr [{b} + 0x1C5D3C]",
        "fadd dword ptr [{b} + 0x1C5D3C]",
        "fmul dword ptr [{b} + 0x213FC8]",
        "fmul dword ptr [{b} + 0x21388C]",
        "fmul dword ptr [{b} + 0x214294]",
        "fmul dword ptr [{b} + 0x214170]",
        "fstp dword ptr [{b} + 0x2D3100]";
        b = base()
    );
    p(RECORD, a(FORCE));
    unsafe { contact_force() }.apply(r);
    unsafe { place_sound_e5() }?;
    copy3(0x6D3018, FORCE);
    fill3(FORCE + 0xC, 0);
    x87_run!(
        "fld dword ptr [{b} + 0x31A0A0]",
        "fadd dword ptr [{b} + 0x1C5D60]",
        "fdiv dword ptr [{b} + 0x1C5D60]",
        "fstp dword ptr [{b} + 0x2D3014]";
        b = base()
    );
    hold_high(0x6D3014, 0x5C5D38);
    p(0x71A160, mul(g(0x6D3014), g(0x71A160)));
    p(0x71AA10, g(0x6D310C));
    Ok(())
}

/// The Rust version.
pub unsafe fn crash_contact(entry: &Regs) -> Result<Left, Refused> {
    {
        let mut r = *entry;
        steps(&mut r)?;
        Ok(left_of(&r))
    }
}
