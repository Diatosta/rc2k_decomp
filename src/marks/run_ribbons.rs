//! `marks::run_ribbons` (0x4CB71A, with its chunk 0x4CF85E .. 0x4CFAAF): the marks' ring
//! walked for its streaks (kind 0x12) when the settings' +0x60 is set: each entry of
//! `g_marksRingCount` from `g_marksRingCursor` along +4; a streak's life (+0x84) less
//! `g_marksFrameTicks` (dies when that borrows, or when its y is below its ground +0x28:
//! `fcomp`); a new one (x 0: no place yet) has its place turned by the view into its last
//! (+0x1C ..) and is moved once; then moved (the motion's y gains `flt_93A572` a tick), its
//! last place into `dword_8EB8E1` .., its place turned into `flt_8EB8D5` .. (kept as the
//! last when the settings' +0xC is set or +0x10 is 1); the first vertex's specular
//! `scene::fog_at` of the last depth's bits; `marks::draw_ribbon` (cell 0x20, width 0x800,
//! the alpha the colour's index, the colour `dword_939DB2`[life times +0x78], or
//! `dword_939BB2`[..] when `dword_765238` is set). A dead entry is moved to just before
//! the cursor (or that moved on past it).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `marks::run_frame` (0x4D1640 .., after `sub_4CB64D`). The x87 blocks are the
//! original's instructions over the entries and the game's variables; the fog its Rust
//! version. Refuses, before anything (the walk simulated), null settings, a null entry or
//! link on the way, and a colour index out of its table.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::emitters::{RING_COUNT_RVA, RING_CURSOR_RVA};
use crate::marks::setup::SETTINGS_RVA;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;
use crate::scene::fog_at::fog_at;

const TICKS_RVA: Slot = Slot::at(0x764FD0); // g_marksFrameTicks
const STREAK: u32 = 0x12;
const LAST_RVA: Slot = Slot::at(0x8EB8E1); // dword_8EB8E1 .. dword_8EB8E9
const PLACE_RVA: Slot = Slot::at(0x8EB8D5); // flt_8EB8D5 .. flt_8EB8DD
const SPECULAR_RVA: usize = rva(0x9605B4); // the first vertex's specular
const INDEX_RVA: Slot = Slot::at(0x9356C0); // dword_9356C0
const OTHER_RVA: Slot = Slot::at(0x765238); // dword_765238
const COLOURS: [(Slot, i32); 2] = [(Slot::at(0x939DB2), 257), (Slot::at(0x939BB2), 128)];

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The original's 0x4CF890 .. 0x4CF94D: a new streak's place turned into its last (+0x1C ..), and a first move.
unsafe fn first_frame(e: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{e} + 0x18]",
            "fld dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x10]",
            "fld dword ptr [{rc} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA2FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{rc} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{rc} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA300]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA304]",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{e} + 0x1C]",
            "fstp dword ptr [{e} + 0x20]",
            "fstp dword ptr [{e} + 0x24]",
            "fld dword ptr [{e} + 0x2C]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x10]",
            "fstp dword ptr [{e} + 0x10]",
            "fld dword ptr [{e} + 0x30]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x14]",
            "fstp dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x34]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x18]",
            "fstp dword ptr [{e} + 0x18]",
            "fld dword ptr [{mk} + 0x1D562E]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x30]",
            "fstp dword ptr [{e} + 0x30]",
            rc = in(reg) crate::race::store::asm_base(),
            mk = in(reg) crate::marks::store::base(),
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4CF94D .. 0x4CF98C: the place moved by the motion, the motion's y by `flt_93A572`.
unsafe fn move_(e: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{e} + 0x2C]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x10]",
            "fstp dword ptr [{e} + 0x10]",
            "fld dword ptr [{e} + 0x30]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x14]",
            "fstp dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x34]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x18]",
            "fstp dword ptr [{e} + 0x18]",
            "fld dword ptr [{mk} + 0x1D5632]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x30]",
            "fstp dword ptr [{e} + 0x30]",
            mk = in(reg) crate::marks::store::base(),
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4CF9A6 .. 0x4CFA2D: the place turned by the view into `flt_8EB8D5` ...
unsafe fn turn_place(e: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{e} + 0x18]",
            "fld dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x10]",
            "fld dword ptr [{rc} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA2FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{rc} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{rc} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA300]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA304]",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{mk} + 0x186995]",
            "fstp dword ptr [{mk} + 0x186999]",
            "fstp dword ptr [{mk} + 0x18699D]",
            rc = in(reg) crate::race::store::asm_base(),
            mk = in(reg) crate::marks::store::base(),
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The colour table in use and its size.
unsafe fn colours() -> (Slot, i32) {
    unsafe { COLOURS[(game_read::<u32>(OTHER_RVA) != 0) as usize] }
}

/// Why it refuses, if it does: the walk as the original makes it, nothing written.
unsafe fn refusal() -> Option<&'static str> {
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Some("no marks settings");
        }
        let ticks: u32 = game_read(TICKS_RVA);
        let mut e: u32 = game_read(RING_CURSOR_RVA);
        let count = game_read::<u32>(RING_COUNT_RVA) as i32;
        if count > 0 && (e == 0 || get(e) == 0) {
            return Some("a null ring entry");
        }
        for _ in 0..count.max(0) {
            if e == 0 || get(e) == 0 || get(e + 4) == 0 {
                return Some("a null ring entry or link");
            }
            if get(s + 0x60) != 0 && get(e + 0xC) == STREAK {
                let life = get(e + 0x84);
                if life >= ticks && x87::compare_status(get(e + 0x14), get(e + 0x28)) != 0x100 {
                    if fog_at(0).is_err() {
                        return Some("the fog table is not there");
                    }
                    let i = x87::int_times_to_int(life.wrapping_sub(ticks) as i32, get(e + 0x78));
                    if !(0..colours().1).contains(&i) {
                        return Some("a streak's colour index past its table");
                    }
                }
            }
            e = get(e + 4);
        }
        None
    }
}

/// The streak step at e (see the header): whether the entry dies.
unsafe fn streak(e: u32) -> Result<bool, Refused> {
    unsafe {
        let t: u32 = game_read(TICKS_RVA);
        let life = get(e + 0x84);
        put(e + 0x84, life.wrapping_sub(t));
        if life < t {
            return Ok(true);
        }
        if x87::compare_status(get(e + 0x14), get(e + 0x28)) == 0x100 {
            return Ok(true);
        }
        if get(e + 0x10) == 0 {
            first_frame(e);
        }
        move_on(e);
        game_write(LAST_RVA, get(e + 0x1C));
        game_write(LAST_RVA + 4, get(e + 0x20));
        game_write(LAST_RVA + 8, get(e + 0x24));
        turn_place(e);
        let s: u32 = game_read(SETTINGS_RVA);
        if get(s + 0xC) != 0 || get(s + 0x10) == 1 {
            put(e + 0x1C, game_read(PLACE_RVA));
            put(e + 0x20, game_read(PLACE_RVA + 4));
            put(e + 0x24, game_read(PLACE_RVA + 8));
        }
        game_write(SPECULAR_RVA, fog_at(get(e + 0x24))?);
        let i = x87::int_times_to_int(get(e + 0x84) as i32, get(e + 0x78));
        game_write(INDEX_RVA, i);
        let colour = game_read::<u32>(colours().0 + 4 * i as usize);
        crate::marks::draw_ribbon::draw_ribbon(0x20, i as u32, 0x800, colour)?;
    }
    Ok(false)
}

unsafe fn move_on(e: u32) {
    unsafe { move_(e) }
}

/// The Rust version (see the header).
pub unsafe fn run_ribbons() -> Result<(), Refused> {
    if let Some(why) = unsafe { refusal() } {
        return Err(BadInput(why).into());
    }
    unsafe {
        let mut e: u32 = game_read(RING_CURSOR_RVA);
        let mut counter = game_read::<u32>(RING_COUNT_RVA) as i32;
        loop {
            counter = counter.wrapping_sub(1);
            if counter < 0 {
                break;
            }
            let next = get(e + 4);
            let settings: u32 = game_read(SETTINGS_RVA);
            if get(settings + 0x60) != 0 && get(e + 0xC) == STREAK && streak(e)? {
                game_write(
                    RING_COUNT_RVA,
                    game_read::<u32>(RING_COUNT_RVA).wrapping_sub(1),
                );
                if e == game_read::<u32>(RING_CURSOR_RVA) {
                    game_write(RING_CURSOR_RVA, get(e + 4));
                } else {
                    let (before, after) = (get(e), get(e + 4));
                    put(before + 4, after);
                    put(after, before);
                    let cursor: u32 = game_read(RING_CURSOR_RVA);
                    let last = get(cursor);
                    put(cursor, e);
                    put(last + 4, e);
                    put(e, last);
                    put(e + 4, cursor);
                }
            }
            e = next;
        }
    }
    Ok(())
}
