//! `marks::run_ring` (0x4CB64D, with its chunk 0x4CE681 .. 0x4CF85E): the emitters' ring's
//! frame. Each of `g_marksRingCount` entries from `g_marksRingCursor` along +4 stepped by
//! its kind (+0xC; others nothing): the life (+0x84) less `g_marksFrameTicks` (dies when
//! that borrows) and, but for the boxed kinds' own test, death below its ground (+0x28,
//! `fcomp`). When the settings' entry flag (`g_marksSettings` + the entry's +8 + 0x60) is
//! set it is drawn: kinds 5, 0xD with their place on the object's (`[[+0x68] + 0x24] +
//! 0x30` .., the sums unrounded), 6, 0x15, 1, 0x13, 0x16 at their place, by
//! `marks::draw_cell_sprite` (0xB by its twin `draw_cell_sprite_2`; cell +0x58, angle
//! +0x50, alpha +0x74, size +0x60 as whole numbers or as they are); 0x10, 0x11 by
//! `marks::draw_subcell_sprite` (subcell +0x58 plus the life's frame of four). Then the
//! angle on (+0x54 times `flt_764FD4`, kept to 12 bits), the place moved by the motion, the
//! kind's gains, dampings (a part that changes sign stops at 0), the size pulled toward
//! +0x70 (kinds 6, 0x15 only while outside the object's box and not facing it; kind 1 also
//! its +0x64 pulled toward `flt_7651DC` when near and facing), the alpha faded by +0x78;
//! kinds 0x10, 0x11 die inside the object's box above `flt_935670`. A dead entry is moved
//! to just before the cursor (or that moved on past it).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as the walk leaves them
//!
//! Caller: `marks::run_frame` (0x4D163B, after `run_ribbons`' set-up). The longer x87 runs
//! are the original's instructions over the entries and the game's variables; the draws
//! through `part` (1 `draw_cell_sprite`, 2 its twin, 3 `draw_subcell_sprite`, with their
//! three values), in the game their Rust versions. Refuses, before anything, null
//! settings, a null entry or link on the way, a null object where one is read, and the
//! draws' own refusals for an entry that may be drawn. Shadow
//! mode runs the original alone (it draws). Self-test: a copy of the whole range (its
//! steps' return address pointed into the copy) whose draws go to stand-ins taking their
//! values off, against the Rust version, on a ring and objects of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::marks::emitters::{RING_COUNT_RVA, RING_CURSOR_RVA};
use crate::marks::setup::SETTINGS_RVA;
use crate::math::x87;
use crate::scene::matrix3::extended;

const TICKS_RVA: usize = rva(0x764FD0); // g_marksFrameTicks
const SPIN_K_RVA: usize = rva(0x764FD4); // flt_764FD4
const WHOLE_RVA: usize = rva(0x9356C0); // dword_9356C0
const BOX_RVA: [usize; 2] = [rva(0x935680), rva(0x935684)]; // dword_935680, dword_935684
const FACING_RVA: usize = rva(0x9356B4); // flt_9356B4
const NEAR_RVA: usize = rva(0x9356BC); // flt_9356BC
const PULL_LIMIT_RVA: usize = rva(0x765140); // dword_765140
const PULL_TO_RVA: usize = rva(0x7651DC); // flt_7651DC
const TOP_RVA: usize = rva(0x935670); // flt_935670
const FRAME_K_RVA: usize = rva(0x76515C); // flt_76515C

/// A call by place (see the header), with its three values (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

fn k(addr: usize) -> u32 {
    unsafe { game_read(rva(addr)) }
}

/// The original's 0x4CE9AF .. 0x4CEA40 (and 0x4CEC77 ..): the place less the object's (`[p + 0x30]` ..) into `flt_935718` .., turned by the object's matrix (`m`), |x| and |z| `fistp`ed into `dword_935680` / `dword_935684`.
unsafe fn box_(b: u32, e: u32, p: u32, m: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{e} + 0x10]",
            "fsub dword ptr [{p} + 0x30]",
            "fstp dword ptr [{b} + 0x535718]",
            "fld dword ptr [{e} + 0x14]",
            "fsub dword ptr [{p} + 0x34]",
            "fstp dword ptr [{b} + 0x53571C]",
            "fld dword ptr [{e} + 0x18]",
            "fsub dword ptr [{p} + 0x38]",
            "fstp dword ptr [{b} + 0x535720]",
            "fld dword ptr [{b} + 0x535720]",
            "fld dword ptr [{b} + 0x53571C]",
            "fld dword ptr [{b} + 0x535718]",
            "fld dword ptr [{m} + 0x0]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{m} + 0x10]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{m} + 0x20]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{m} + 0x4]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{m} + 0x14]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{m} + 0x24]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{m} + 0x18]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{m} + 0x28]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCA",
            ".byte 0xD9, 0xE1",
            "fistp dword ptr [{b} + 0x535680]",
            ".byte 0xDD, 0xC0",
            ".byte 0xD9, 0xF7",
            ".byte 0xD9, 0xE1",
            "fistp dword ptr [{b} + 0x535684]",
            b = in(reg) b,
            e = in(reg) e,
            p = in(reg) p,
            m = in(reg) m,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4CEA5E .. 0x4CEAD4 (and 0x4CED26 ..): that way made one long (`flt_76516C` over its length), dotted with the object's +0x4C .. into `flt_9356B4`.
unsafe fn facing(b: u32, o: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x535718]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x53571C]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x535720]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{b} + 0x36516C]",
            "fld dword ptr [{b} + 0x535718]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{b} + 0x535718]",
            "fld dword ptr [{b} + 0x53571C]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{b} + 0x53571C]",
            "fld dword ptr [{b} + 0x535720]",
            ".byte 0xDE, 0xC9",
            "fstp dword ptr [{b} + 0x535720]",
            "fld dword ptr [{b} + 0x535718]",
            "fmul dword ptr [{o} + 0x4C]",
            "fld dword ptr [{b} + 0x53571C]",
            "fmul dword ptr [{o} + 0x50]",
            "fld dword ptr [{b} + 0x535720]",
            "fmul dword ptr [{o} + 0x54]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{b} + 0x5356B4]",
            b = in(reg) b,
            o = in(reg) o,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4CEF8A .. 0x4CEFDA: the place less the object's into `flt_935718` .., its length (kept, 80 bits, at `l`) less `flt_93A526` into `flt_9356BC`.
unsafe fn pull_length(b: u32, e: u32, p: u32, l: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{e} + 0x10]",
            "fsub dword ptr [{p} + 0x30]",
            "fstp dword ptr [{b} + 0x535718]",
            "fld dword ptr [{e} + 0x14]",
            "fsub dword ptr [{p} + 0x34]",
            "fstp dword ptr [{b} + 0x53571C]",
            "fld dword ptr [{e} + 0x18]",
            "fsub dword ptr [{p} + 0x38]",
            "fstp dword ptr [{b} + 0x535720]",
            "fld dword ptr [{b} + 0x535718]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x53571C]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x535720]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{b} + 0x53A526]",
            "fstp dword ptr [{b} + 0x5356BC]",
            "fstp tbyte ptr [{l}]",
            b = in(reg) b,
            e = in(reg) e,
            p = in(reg) p,
            l = in(reg) l,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4CEFEC .. 0x4CF07F: the way made one long by the kept length, the object's +0x4C .. over its +0x88, minus their dot into `flt_9356B4`.
unsafe fn pull_facing(b: u32, o: u32, l: u32) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{l}]",
            "fdivr dword ptr [{b} + 0x36516C]",
            "fld dword ptr [{b} + 0x535718]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{b} + 0x535718]",
            "fld dword ptr [{b} + 0x53571C]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{b} + 0x53571C]",
            "fld dword ptr [{b} + 0x535720]",
            ".byte 0xDE, 0xC9",
            "fstp dword ptr [{b} + 0x535720]",
            "fild dword ptr [{o} + 0x88]",
            "fdivr dword ptr [{b} + 0x36516C]",
            "fld dword ptr [{o} + 0x4C]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{b} + 0x535724]",
            "fld dword ptr [{o} + 0x50]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{b} + 0x535728]",
            "fld dword ptr [{o} + 0x54]",
            ".byte 0xDE, 0xC9",
            "fstp dword ptr [{b} + 0x53572C]",
            "fld dword ptr [{b} + 0x535718]",
            "fmul dword ptr [{b} + 0x535724]",
            "fld dword ptr [{b} + 0x53571C]",
            "fmul dword ptr [{b} + 0x535728]",
            "fld dword ptr [{b} + 0x535720]",
            "fmul dword ptr [{b} + 0x53572C]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xE0",
            "fstp dword ptr [{b} + 0x5356B4]",
            b = in(reg) b,
            o = in(reg) o,
            l = in(reg) l,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4CF08B .. 0x4CF0D8: the entry's +0x64 pulled toward `flt_7651DC` (`fist` into `dword_9356C0`).
unsafe fn pull_strength(b: u32, e: u32, o: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{o} + 0x88]",
            "fmul dword ptr [{b} + 0x53A52A]",
            "fadd dword ptr [{b} + 0x36516C]",
            "fdivr dword ptr [{b} + 0x36516C]",
            "fsubr dword ptr [{b} + 0x36516C]",
            "fstp dword ptr [{b} + 0x5356C0]",
            "fld dword ptr [{b} + 0x3651DC]",
            "fsub dword ptr [{e} + 0x64]",
            "fld dword ptr [{b} + 0x53A52E]",
            "fmul dword ptr [{b} + 0x5356C0]",
            ".byte 0xDE, 0xC9",
            "fimul dword ptr [{b} + 0x364FD0]",
            "fadd dword ptr [{e} + 0x64]",
            "fist dword ptr [{b} + 0x5356C0]",
            "fstp dword ptr [{e} + 0x64]",
            b = in(reg) b,
            e = in(reg) e,
            o = in(reg) o,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4CF65C .. 0x4CF6B7 (and 0x4CF7C4 ..): the place turned by the object's matrix (`m`), |x| and |z| into `dword_935680` / `dword_935684`.
unsafe fn box_place(b: u32, e: u32, m: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{e} + 0x18]",
            "fld dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x10]",
            "fld dword ptr [{m} + 0x0]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{m} + 0x10]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{m} + 0x20]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{m} + 0x4]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{m} + 0x14]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{m} + 0x24]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{m} + 0x18]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{m} + 0x28]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCA",
            ".byte 0xD9, 0xE1",
            "fistp dword ptr [{b} + 0x535680]",
            ".byte 0xDD, 0xC0",
            ".byte 0xD9, 0xF7",
            ".byte 0xD9, 0xE1",
            "fistp dword ptr [{b} + 0x535684]",
            b = in(reg) b,
            e = in(reg) e,
            m = in(reg) m,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The kinds' constants: (the y gain, the three dampings, the size's rate).
struct Kind {
    gain: usize,
    damp: [usize; 3],
    pull: usize,
}

const K6: Kind = Kind {
    gain: 0,
    damp: [0x93A4F2, 0x93A4F6, 0x93A4FA],
    pull: 0x93A4FE,
};
const K15: Kind = Kind {
    gain: 0,
    damp: [0x93A502, 0x93A506, 0x93A50A],
    pull: 0x93A50E,
};
const K1: Kind = Kind {
    gain: 0,
    damp: [0x93A516, 0x93A51A, 0x93A51E],
    pull: 0x93A522,
};
const KD: Kind = Kind {
    gain: 0x93A532,
    damp: [0x93A536, 0x93A53A, 0x93A53E],
    pull: 0x93A542,
};
const K13: Kind = Kind {
    gain: 0x93A552,
    damp: [0x93A546, 0x93A54A, 0x93A54E],
    pull: 0,
};
const K16: Kind = Kind {
    gain: 0x93A562,
    damp: [0x93A556, 0x93A55A, 0x93A55E],
    pull: 0,
};

/// Why it refuses, if it does: the walk's pointers checked, nothing written.
unsafe fn refusal() -> Option<&'static str> {
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Some("no marks settings");
        }
        let mut e: u32 = game_read(RING_CURSOR_RVA);
        let count = game_read::<u32>(RING_COUNT_RVA) as i32;
        if count > 0 && (e == 0 || get(e) == 0) {
            return Some("a null ring entry");
        }
        for _ in 0..count.max(0) {
            if e == 0 || get(e) == 0 || get(e + 4) == 0 {
                return Some("a null ring entry or link");
            }
            let kind = get(e + 0xC);
            if [5, 0xB, 6, 0x15, 1, 0xD, 0x13, 0x16, 0x10, 0x11].contains(&kind) {
                let obj = get(e + 0x68);
                let needs = matches!(kind, 6 | 0x15 | 1 | 0x10 | 0x11)
                    || get(s.wrapping_add(get(e + 8)).wrapping_add(0x60)) != 0;
                if needs && (obj == 0 || get(obj + 0x24) == 0) {
                    return Some("a ring entry without its object");
                }
                // The draws' own refusals, for an entry that may be drawn.
                if get(s.wrapping_add(get(e + 8)).wrapping_add(0x60)) != 0 {
                    if crate::scene::fog_at::fog_at(0).is_err() {
                        return Some("the fog table is not there");
                    }
                    let bad = if matches!(kind, 0x10 | 0x11) {
                        get(e + 0x58).wrapping_add(3) >= 0x410
                    } else {
                        get(e + 0x58) > 0x40 || get(e + 0x50) >= 0x1000
                    };
                    if bad {
                        return Some("a ring entry's cell or angle past its table");
                    }
                }
            }
            e = get(e + 4);
        }
        None
    }
}

/// The life and the ground: whether the entry dies (eax -1).
unsafe fn aged(g: &mut Regs, e: u32, ground: bool) -> bool {
    unsafe {
        g.eax = game_read(TICKS_RVA);
        let life = get(e + 0x84);
        put(e + 0x84, life.wrapping_sub(g.eax));
        if life < g.eax {
            g.eax = 0xFFFF_FFFF;
            return true;
        }
        if ground {
            g.eax = u32::from(x87::compare_status(get(e + 0x14), get(e + 0x28)));
            if g.eax == 0x100 {
                g.eax = 0xFFFF_FFFF;
                return true;
            }
        }
        false
    }
}

/// The place on the object's (unrounded sums), st(0) first.
unsafe fn on_object(e: u32) -> [[u8; 10]; 3] {
    unsafe {
        let m = get(get(e + 0x68) + 0x24);
        [
            x87::add_80(get(e + 0x10), get(m + 0x30)),
            x87::add_80(get(e + 0x14), get(m + 0x34)),
            x87::add_80(get(e + 0x18), get(m + 0x38)),
        ]
    }
}

unsafe fn at_place(e: u32) -> [[u8; 10]; 3] {
    unsafe {
        [
            extended(get(e + 0x10)),
            extended(get(e + 0x14)),
            extended(get(e + 0x18)),
        ]
    }
}

/// The settings' flag for the entry.
unsafe fn shown(g: &mut Regs, e: u32) -> bool {
    unsafe {
        g.eax = get(e + 8);
        g.ebx = game_read(SETTINGS_RVA);
        get(g.eax.wrapping_add(g.ebx).wrapping_add(0x60)) != 0
    }
}

/// The alpha (top byte) and the size as whole numbers (`fistp` through `dword_9356C0`).
unsafe fn whole_alpha(g: &mut Regs, e: u32) {
    unsafe {
        let a = x87::to_int(get(e + 0x74));
        game_write(WHOLE_RVA, a);
        g.ecx = (a as u32) << 24;
    }
}

unsafe fn whole(g: &mut Regs, v: i32) {
    unsafe {
        game_write(WHOLE_RVA, v);
        g.edx = v as u32;
    }
}

/// The angle on and the place moved.
unsafe fn spin_and_move(g: &mut Regs, e: u32, spin: bool) {
    unsafe {
        let t: i32 = game_read(TICKS_RVA);
        if spin {
            g.eax = x87::int_times_to_int(get(e + 0x54) as i32, game_read(SPIN_K_RVA)) as u32;
            put(e + 0x50, get(e + 0x50).wrapping_add(g.eax) & 0xFFF);
        }
        for o in [0x10u32, 0x14, 0x18] {
            put(e + o, x87::plus_times_int(get(e + o), get(e + o + 0x1C), t));
        }
    }
}

unsafe fn gain_y(e: u32, k_addr: usize) {
    unsafe {
        let t: i32 = game_read(TICKS_RVA);
        put(e + 0x30, x87::plus_times_int(get(e + 0x30), k(k_addr), t));
    }
}

/// The three dampings (a part that changes sign stops at 0).
unsafe fn damp3(g: &mut Regs, e: u32, ks: [usize; 3]) {
    unsafe {
        let t: i32 = game_read(TICKS_RVA);
        for (o, kk) in [0x2Cu32, 0x30, 0x34].into_iter().zip(ks) {
            let old = get(e + o);
            game_write(WHOLE_RVA, old);
            let new = x87::damp(old, k(kk), t);
            put(e + o, new);
            g.eax = (old ^ new).wrapping_add(old ^ new);
            if (old ^ new) & 0x8000_0000 != 0 {
                put(e + o, 0);
            }
        }
    }
}

/// The size pulled toward +0x70.
unsafe fn pull_size(g: &mut Regs, e: u32, k_addr: usize) {
    unsafe {
        let t: i32 = game_read(TICKS_RVA);
        let (w, f) = x87::approach(get(e + 0x70) as i32, get(e + 0x60), k(k_addr), t);
        game_write(WHOLE_RVA, w);
        put(e + 0x60, f);
        g.eax = w as u32;
        if w > get(e + 0x70) as i32 {
            put(e + 0x60, x87::int_to_float(get(e + 0x70) as i32));
        }
    }
}

unsafe fn fade(e: u32) {
    unsafe {
        let t: i32 = game_read(TICKS_RVA);
        put(
            e + 0x74,
            x87::less_times_int(get(e + 0x74), get(e + 0x78), t),
        );
    }
}

/// The box test of kinds 6 and 0x15: whether the size is pulled.
unsafe fn outside_and_away(g: &mut Regs, e: u32) -> bool {
    unsafe {
        let b = game_addr(0) as u32;
        let obj = get(e + 0x68);
        g.ebp = get(obj + 0x24);
        g.eax = obj + 0x94;
        box_(b, e, g.ebp, g.eax);
        g.eax = get(e + 0x7C);
        let mut outside = (g.eax as i32) < game_read::<i32>(BOX_RVA[0]);
        if !outside {
            g.eax = get(e + 0x80);
            outside = (g.eax as i32) < game_read::<i32>(BOX_RVA[1]);
        }
        if !outside {
            return false;
        }
        g.ebp = obj;
        facing(b, obj);
        game_read::<i32>(FACING_RVA) <= 0
    }
}

/// Kinds 5 and 0xB (part 1 or 2; their y gain, size and alpha gains).
unsafe fn drift_kind(
    g: &mut Regs,
    e: u32,
    part: &mut Part,
    place: u32,
    ks: [usize; 3],
) -> Result<(), Refused> {
    unsafe {
        if aged(g, e, true) {
            return Ok(());
        }
        if shown(g, e) {
            g.eax = get(get(e + 0x68) + 0x24);
            let p = on_object(e);
            [g.eax, g.ebx, g.ecx, g.edx] =
                [get(e + 0x58), get(e + 0x50), get(e + 0x74), get(e + 0x60)];
            part(place, g, &p)?;
        }
        spin_and_move(g, e, true);
        gain_y(e, ks[0]);
        let t: i32 = game_read(TICKS_RVA);
        put(
            e + 0x60,
            x87::times_int_plus_int_to_int(k(ks[1]), t, get(e + 0x60) as i32) as u32,
        );
        put(
            e + 0x74,
            x87::int_times_int_plus_int(k(ks[2]) as i32, t, get(e + 0x74) as i32) as u32,
        );
        g.eax = 0;
    }
    Ok(())
}

/// Kinds 6, 0x15, 1, 0xD, 0x13, 0x16 (drawn by part 1).
unsafe fn fading_kind(
    g: &mut Regs,
    e: u32,
    part: &mut Part,
    kind: u32,
    c: &Kind,
) -> Result<(), Refused> {
    unsafe {
        if aged(g, e, true) {
            return Ok(());
        }
        if (get(e + 0x74) as i32) < 0 {
            put(e + 0x74, 0);
        }
        if matches!(kind, 0x13 | 0x16) {
            g.ebp = get(e + 0x68);
        }
        if shown(g, e) {
            if kind == 1 {
                g.ebp = get(e + 0x68);
            }
            let p = if kind == 0xD {
                g.eax = get(get(e + 0x68) + 0x24);
                on_object(e)
            } else {
                at_place(e)
            };
            [g.eax, g.ebx] = [get(e + 0x58), get(e + 0x50)];
            whole_alpha(g, e);
            let size = if kind == 1 {
                x87::plus_product_to_int(get(e + 0x60), get(e + 0x64), k(0x93A512))
            } else {
                x87::to_int(get(e + 0x60))
            };
            whole(g, size);
            part(1, g, &p)?;
        }
        spin_and_move(g, e, true);
        if c.gain != 0 && kind == 0xD {
            gain_y(e, c.gain);
        }
        damp3(g, e, c.damp);
        match kind {
            6 | 0x15 => {
                if outside_and_away(g, e) {
                    pull_size(g, e, c.pull);
                }
                fade(e);
            }
            1 => {
                pull_size(g, e, c.pull);
                fade(e);
                pull_toward(g, e);
            }
            0xD => {
                pull_size(g, e, c.pull);
                fade(e);
            }
            _ => {
                fade(e);
                let t: i32 = game_read(TICKS_RVA);
                put(e + 0x60, x87::plus_times_int(get(e + 0x60), k(c.gain), t));
            }
        }
        g.eax = 0;
    }
    Ok(())
}

/// Kind 1's pull of its +0x64 when near and facing.
unsafe fn pull_toward(g: &mut Regs, e: u32) {
    unsafe {
        let b = game_addr(0) as u32;
        let obj = get(e + 0x68);
        g.eax = obj;
        g.ebp = get(obj + 0x24);
        let mut length = [0u8; 10];
        let l = length.as_mut_ptr() as u32;
        pull_length(b, e, g.ebp, l);
        if game_read::<i32>(NEAR_RVA) < 0 {
            return;
        }
        g.ebp = obj;
        pull_facing(b, obj, l);
        if game_read::<i32>(FACING_RVA) <= 0 {
            return;
        }
        pull_strength(b, e, obj);
        g.eax = game_read(WHOLE_RVA);
        if (g.eax as i32) > game_read::<i32>(PULL_LIMIT_RVA) {
            put(e + 0x64, x87::loaded(game_read(PULL_TO_RVA)));
        }
    }
}

/// Kinds 0x10, 0x11 (part 3): boxed, no ground test of their own first.
unsafe fn boxed_kind(
    g: &mut Regs,
    e: u32,
    part: &mut Part,
    gain: usize,
    both: bool,
) -> Result<(), Refused> {
    unsafe {
        if aged(g, e, true) {
            return Ok(());
        }
        spin_and_move(g, e, false);
        gain_y(e, gain);
        if shown(g, e) {
            g.eax = get(get(e + 0x68) + 0x24);
            let p = on_object(e);
            g.edx = get(e + 0x60);
            let f = x87::int_times_times_to_int(
                get(e + 0x84) as i32,
                game_read(FRAME_K_RVA),
                get(e + 0x5C),
            );
            game_write(WHOLE_RVA, f);
            g.eax = ((f as u32) & 3).wrapping_add(get(e + 0x58));
            part(3, g, &p)?;
        }
        let b = game_addr(0) as u32;
        g.eax = get(e + 0x68) + 0x94;
        box_place(b, e, g.eax);
        g.eax = get(e + 0x7C);
        let mut outside = (g.eax as i32) < game_read::<i32>(BOX_RVA[0]);
        if !outside && both {
            g.eax = get(e + 0x80);
            outside = (g.eax as i32) < game_read::<i32>(BOX_RVA[1]);
        }
        if !outside {
            g.eax = u32::from(x87::compare_status(get(e + 0x14), game_read(TOP_RVA)));
            if g.eax == 0 {
                g.eax = 0xFFFF_FFFF;
                return Ok(());
            }
        }
        g.eax = u32::from(x87::compare_status(get(e + 0x14), get(e + 0x28)));
        g.eax = if g.eax == 0x100 { 0xFFFF_FFFF } else { 0 };
    }
    Ok(())
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_ring(entry: &Regs, mut part: Part) -> Result<Regs, Refused> {
    if let Some(why) = unsafe { refusal() } {
        return Err(BadInput(why).into());
    }
    let mut g = *entry;
    unsafe {
        g.esi = game_read(RING_CURSOR_RVA);
        let mut counter = game_read::<u32>(RING_COUNT_RVA) as i32;
        loop {
            counter = counter.wrapping_sub(1);
            if counter < 0 {
                break;
            }
            let e = g.esi;
            let next = get(e + 4);
            match get(e + 0xC) {
                5 => drift_kind(&mut g, e, &mut part, 1, [0x93A4DA, 0x93A4DE, 0x93A4E2])?,
                0xB => drift_kind(&mut g, e, &mut part, 2, [0x93A4E6, 0x93A4EA, 0x93A4EE])?,
                6 => fading_kind(&mut g, e, &mut part, 6, &K6)?,
                0x15 => fading_kind(&mut g, e, &mut part, 0x15, &K15)?,
                1 => fading_kind(&mut g, e, &mut part, 1, &K1)?,
                0xD => fading_kind(&mut g, e, &mut part, 0xD, &KD)?,
                0x13 => fading_kind(&mut g, e, &mut part, 0x13, &K13)?,
                0x16 => fading_kind(&mut g, e, &mut part, 0x16, &K16)?,
                0x10 => boxed_kind(&mut g, e, &mut part, 0x93A566, true)?,
                0x11 => boxed_kind(&mut g, e, &mut part, 0x93A56A, false)?,
                _ => g.eax = 0,
            }
            g.esi = e;
            if g.eax != 0 {
                game_write(
                    RING_COUNT_RVA,
                    game_read::<u32>(RING_COUNT_RVA).wrapping_sub(1),
                );
                if e == game_read::<u32>(RING_CURSOR_RVA) {
                    g.eax = get(e + 4);
                    game_write(RING_CURSOR_RVA, g.eax);
                } else {
                    g.ebx = get(e);
                    g.ecx = get(e + 4);
                    put(g.ebx + 4, g.ecx);
                    put(g.ecx, g.ebx);
                    g.ebx = game_read(RING_CURSOR_RVA);
                    g.ecx = get(g.ebx);
                    put(g.ebx, e);
                    put(g.ecx + 4, e);
                    put(e, g.ecx);
                    put(e + 4, g.ebx);
                }
            }
            g.esi = next;
        }
    }
    Ok(g)
}

/// In the game: the draws' Rust versions.
pub fn game_part(place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::marks::{draw_cell_sprite as cell, draw_subcell_sprite as sub};
    let p = [v[0], v[1], v[2]];
    let left = unsafe {
        match place {
            3 => sub::draw_subcell_sprite(r, p)?,
            _ => cell::draw_cell_sprite(r, p)?,
        }
    };
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
