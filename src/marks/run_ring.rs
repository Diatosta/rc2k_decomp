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
//! Out: nothing (its one caller reads no register after it)
//!
//! Caller: `marks::run_frame` (0x4D163B, after `run_ribbons`' set-up). The longer x87 runs
//! are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the entries and the game's variables. Refuses,
//! before anything, null settings, a null entry or link on the way, a null object where one
//! is read, and the draws' own refusals for an entry that may be drawn.

use crate::errors::{BadInput, Refused};
use crate::marks::emitters::{RING_COUNT_RVA, RING_CURSOR_RVA};
use crate::marks::setup::SETTINGS_RVA;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;
use crate::scene::matrix3::extended;

const TICKS_RVA: Slot = Slot::at(0x764FD0); // g_marksFrameTicks
const SPIN_K_RVA: Slot = Slot::at(0x764FD4); // flt_764FD4
const WHOLE_RVA: Slot = Slot::at(0x9356C0); // dword_9356C0
const BOX_RVA: [Slot; 2] = [Slot::at(0x935680), Slot::at(0x935684)]; // dword_935680, dword_935684
const FACING_RVA: Slot = Slot::at(0x9356B4); // flt_9356B4
const NEAR_RVA: Slot = Slot::at(0x9356BC); // flt_9356BC
const PULL_LIMIT_RVA: Slot = Slot::at(0x765140); // dword_765140
const PULL_TO_RVA: Slot = Slot::at(0x7651DC); // flt_7651DC
const TOP_RVA: Slot = Slot::at(0x935670); // flt_935670
const FRAME_K_RVA: Slot = Slot::at(0x76515C); // flt_76515C

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

fn k(place: Slot) -> u32 {
    unsafe { game_read(place) }
}

/// The original's 0x4CE9AF .. 0x4CEA40 (and 0x4CEC77 ..): the place less the object's (`[p + 0x30]` ..) into `flt_935718` .., turned by the object's matrix (`m`), |x| and |z| `fistp`ed into `dword_935680` / `dword_935684`.
unsafe fn box_(e: u32, p: u32, m: u32) {
    unsafe {
        box_x87(
            crate::marks::store::base() as usize,
            e as usize,
            p as usize,
            m as usize,
        )
    }
}

/// The original's 0x4CEA5E .. 0x4CEAD4 (and 0x4CED26 ..): that way made one long (`flt_76516C` over its length), dotted with the object's +0x4C .. into `flt_9356B4`.
unsafe fn facing(o: u32) {
    unsafe { facing_x87(crate::marks::store::base() as usize, o as usize) }
}

/// The original's 0x4CEF8A .. 0x4CEFDA: the place less the object's into `flt_935718` .., its length (kept, 80 bits, at `l`) less `flt_93A526` into `flt_9356BC`.
unsafe fn pull_length(e: u32, p: u32, l: u32) {
    unsafe {
        pull_length_x87(
            crate::marks::store::base() as usize,
            e as usize,
            p as usize,
            l as usize,
        )
    }
}

/// The original's 0x4CEFEC .. 0x4CF07F: the way made one long by the kept length, the object's +0x4C .. over its +0x88, minus their dot into `flt_9356B4`.
unsafe fn pull_facing(o: u32, l: u32) {
    unsafe { pull_facing_x87(crate::marks::store::base() as usize, o as usize, l as usize) }
}

/// The original's 0x4CF08B .. 0x4CF0D8: the entry's +0x64 pulled toward `flt_7651DC` (`fist` into `dword_9356C0`).
unsafe fn pull_strength(e: u32, o: u32) {
    unsafe { pull_strength_x87(crate::marks::store::base() as usize, e as usize, o as usize) }
}

/// The original's 0x4CF65C .. 0x4CF6B7 (and 0x4CF7C4 ..): the place turned by the object's matrix (`m`), |x| and |z| into `dword_935680` / `dword_935684`.
unsafe fn box_place(e: u32, m: u32) {
    unsafe { box_place_x87(crate::marks::store::base() as usize, e as usize, m as usize) }
}

/// The kinds' constants: (the y gain, the three dampings, the size's rate).
struct Kind {
    gain: Option<Slot>,
    damp: [Slot; 3],
    pull: Option<Slot>,
}

impl Kind {
    /// The size's rate (the kinds that pull the size have one; the original would read
    /// address 0 for one that has none: refused).
    fn pull(&self) -> Result<Slot, BadInput> {
        self.pull
            .ok_or(BadInput("a ring kind that pulls the size without a rate"))
    }

    /// The y gain (the kinds that gain have one; as `pull`).
    fn gain(&self) -> Result<Slot, BadInput> {
        self.gain
            .ok_or(BadInput("a ring kind that gains without a gain"))
    }
}

const K6: Kind = Kind {
    gain: None,
    damp: [Slot::at(0x93A4F2), Slot::at(0x93A4F6), Slot::at(0x93A4FA)],
    pull: Some(Slot::at(0x93A4FE)),
};
const K15: Kind = Kind {
    gain: None,
    damp: [Slot::at(0x93A502), Slot::at(0x93A506), Slot::at(0x93A50A)],
    pull: Some(Slot::at(0x93A50E)),
};
const K1: Kind = Kind {
    gain: None,
    damp: [Slot::at(0x93A516), Slot::at(0x93A51A), Slot::at(0x93A51E)],
    pull: Some(Slot::at(0x93A522)),
};
const KD: Kind = Kind {
    gain: Some(Slot::at(0x93A532)),
    damp: [Slot::at(0x93A536), Slot::at(0x93A53A), Slot::at(0x93A53E)],
    pull: Some(Slot::at(0x93A542)),
};
const K13: Kind = Kind {
    gain: Some(Slot::at(0x93A552)),
    damp: [Slot::at(0x93A546), Slot::at(0x93A54A), Slot::at(0x93A54E)],
    pull: None,
};
const K16: Kind = Kind {
    gain: Some(Slot::at(0x93A562)),
    damp: [Slot::at(0x93A556), Slot::at(0x93A55A), Slot::at(0x93A55E)],
    pull: None,
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

/// The life and the ground: whether the entry dies.
unsafe fn aged(e: u32, ground: bool) -> bool {
    unsafe {
        let t: u32 = game_read(TICKS_RVA);
        let life = get(e + 0x84);
        put(e + 0x84, life.wrapping_sub(t));
        if life < t {
            return true;
        }
        ground && x87::compare_status(get(e + 0x14), get(e + 0x28)) == 0x100
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
unsafe fn shown(e: u32) -> bool {
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        get(get(e + 8).wrapping_add(settings).wrapping_add(0x60)) != 0
    }
}

/// The alpha (top byte) as a whole number (`fistp` through `dword_9356C0`).
unsafe fn whole_alpha(e: u32) -> u32 {
    unsafe {
        let a = x87::to_int(get(e + 0x74));
        game_write(WHOLE_RVA, a);
        (a as u32) << 24
    }
}

/// The size as a whole number (through `dword_9356C0`).
unsafe fn whole(v: i32) -> u32 {
    unsafe {
        game_write(WHOLE_RVA, v);
        v as u32
    }
}

/// The angle on and the place moved.
unsafe fn spin_and_move(e: u32, spin: bool) {
    unsafe {
        let t: i32 = game_read(TICKS_RVA);
        if spin {
            let turn = x87::int_times_to_int(get(e + 0x54) as i32, game_read(SPIN_K_RVA)) as u32;
            put(e + 0x50, get(e + 0x50).wrapping_add(turn) & 0xFFF);
        }
        for o in [0x10u32, 0x14, 0x18] {
            put(e + o, x87::plus_times_int(get(e + o), get(e + o + 0x1C), t));
        }
    }
}

unsafe fn gain_y(e: u32, k_addr: Slot) {
    unsafe {
        let t: i32 = game_read(TICKS_RVA);
        put(e + 0x30, x87::plus_times_int(get(e + 0x30), k(k_addr), t));
    }
}

/// The three dampings (a part that changes sign stops at 0).
unsafe fn damp3(e: u32, ks: [Slot; 3]) {
    unsafe {
        let t: i32 = game_read(TICKS_RVA);
        for (o, kk) in [0x2Cu32, 0x30, 0x34].into_iter().zip(ks) {
            let old = get(e + o);
            game_write(WHOLE_RVA, old);
            let new = x87::damp(old, k(kk), t);
            put(e + o, new);
            if (old ^ new) & 0x8000_0000 != 0 {
                put(e + o, 0);
            }
        }
    }
}

/// The size pulled toward +0x70.
unsafe fn pull_size(e: u32, k_addr: Slot) {
    unsafe {
        let t: i32 = game_read(TICKS_RVA);
        let (w, f) = x87::approach(get(e + 0x70) as i32, get(e + 0x60), k(k_addr), t);
        game_write(WHOLE_RVA, w);
        put(e + 0x60, f);
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
unsafe fn outside_and_away(e: u32) -> bool {
    unsafe {
        let obj = get(e + 0x68);
        box_(e, get(obj + 0x24), obj + 0x94);
        let outside = (get(e + 0x7C) as i32) < game_read::<i32>(BOX_RVA[0])
            || (get(e + 0x80) as i32) < game_read::<i32>(BOX_RVA[1]);
        if !outside {
            return false;
        }
        facing(obj);
        game_read::<i32>(FACING_RVA) <= 0
    }
}

/// The draws: 1 `draw_cell_sprite`, 2 its twin (the same Rust version), 3
/// `draw_subcell_sprite`; `cell` is the (sub)cell, `size` the size.
unsafe fn draw(
    place: u32,
    cell: u32,
    angle: u32,
    alpha: u32,
    size: u32,
    e: u32,
    p: [[u8; 10]; 3],
) -> Result<(), Refused> {
    use crate::marks::{draw_cell_sprite as c, draw_subcell_sprite as s};
    unsafe {
        match place {
            3 => s::draw_subcell_sprite(cell, size, e, p),
            _ => c::draw_cell_sprite(cell, angle, alpha, size, e, p),
        }
    }
}

/// Kinds 5 and 0xB (draw 1 or 2; their y gain, size and alpha gains): whether it dies.
unsafe fn drift_kind(e: u32, place: u32, ks: [Slot; 3]) -> Result<bool, Refused> {
    unsafe {
        if aged(e, true) {
            return Ok(true);
        }
        if shown(e) {
            let p = on_object(e);
            let [cell, angle, alpha, size] =
                [get(e + 0x58), get(e + 0x50), get(e + 0x74), get(e + 0x60)];
            draw(place, cell, angle, alpha, size, e, p)?;
        }
        spin_and_move(e, true);
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
    }
    Ok(false)
}

/// Kinds 6, 0x15, 1, 0xD, 0x13, 0x16 (draw 1): whether it dies.
unsafe fn fading_kind(e: u32, kind: u32, c: &Kind) -> Result<bool, Refused> {
    unsafe {
        if aged(e, true) {
            return Ok(true);
        }
        if (get(e + 0x74) as i32) < 0 {
            put(e + 0x74, 0);
        }
        if shown(e) {
            let p = if kind == 0xD {
                on_object(e)
            } else {
                at_place(e)
            };
            let [cell, angle] = [get(e + 0x58), get(e + 0x50)];
            let alpha = whole_alpha(e);
            let size = if kind == 1 {
                x87::plus_product_to_int(get(e + 0x60), get(e + 0x64), k(Slot::at(0x93A512)))
            } else {
                x87::to_int(get(e + 0x60))
            };
            let size = whole(size);
            draw(1, cell, angle, alpha, size, e, p)?;
        }
        spin_and_move(e, true);
        if c.gain.is_some() && kind == 0xD {
            gain_y(e, c.gain()?);
        }
        damp3(e, c.damp);
        match kind {
            6 | 0x15 => {
                if outside_and_away(e) {
                    pull_size(e, c.pull()?);
                }
                fade(e);
            }
            1 => {
                pull_size(e, c.pull()?);
                fade(e);
                pull_toward(e);
            }
            0xD => {
                pull_size(e, c.pull()?);
                fade(e);
            }
            _ => {
                fade(e);
                let t: i32 = game_read(TICKS_RVA);
                put(
                    e + 0x60,
                    x87::plus_times_int(get(e + 0x60), k(c.gain()?), t),
                );
            }
        }
    }
    Ok(false)
}

/// Kind 1's pull of its +0x64 when near and facing.
unsafe fn pull_toward(e: u32) {
    unsafe {
        let obj = get(e + 0x68);
        let mut length = [0u8; 10];
        let l = length.as_mut_ptr() as u32;
        pull_length(e, get(obj + 0x24), l);
        if game_read::<i32>(NEAR_RVA) < 0 {
            return;
        }
        pull_facing(obj, l);
        if game_read::<i32>(FACING_RVA) <= 0 {
            return;
        }
        pull_strength(e, obj);
        if game_read::<i32>(WHOLE_RVA) > game_read::<i32>(PULL_LIMIT_RVA) {
            put(e + 0x64, x87::loaded(game_read(PULL_TO_RVA)));
        }
    }
}

/// Kinds 0x10, 0x11 (draw 3): boxed, no ground test of their own first. Whether it dies.
unsafe fn boxed_kind(e: u32, gain: Slot, both: bool) -> Result<bool, Refused> {
    unsafe {
        if aged(e, true) {
            return Ok(true);
        }
        spin_and_move(e, false);
        gain_y(e, gain);
        if shown(e) {
            let p = on_object(e);
            let size = get(e + 0x60);
            let f = x87::int_times_times_to_int(
                get(e + 0x84) as i32,
                game_read(FRAME_K_RVA),
                get(e + 0x5C),
            );
            game_write(WHOLE_RVA, f);
            let subcell = ((f as u32) & 3).wrapping_add(get(e + 0x58));
            draw(3, subcell, 0, 0, size, e, p)?;
        }
        box_place(e, get(e + 0x68) + 0x94);
        let mut outside = (get(e + 0x7C) as i32) < game_read::<i32>(BOX_RVA[0]);
        if !outside && both {
            outside = (get(e + 0x80) as i32) < game_read::<i32>(BOX_RVA[1]);
        }
        if !outside && x87::compare_status(get(e + 0x14), game_read(TOP_RVA)) == 0 {
            return Ok(true);
        }
        Ok(x87::compare_status(get(e + 0x14), get(e + 0x28)) == 0x100)
    }
}

/// The Rust version (see the header).
pub unsafe fn run_ring() -> Result<(), Refused> {
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
            let dies = match get(e + 0xC) {
                5 => drift_kind(
                    e,
                    1,
                    [Slot::at(0x93A4DA), Slot::at(0x93A4DE), Slot::at(0x93A4E2)],
                )?,
                0xB => drift_kind(
                    e,
                    2,
                    [Slot::at(0x93A4E6), Slot::at(0x93A4EA), Slot::at(0x93A4EE)],
                )?,
                6 => fading_kind(e, 6, &K6)?,
                0x15 => fading_kind(e, 0x15, &K15)?,
                1 => fading_kind(e, 1, &K1)?,
                0xD => fading_kind(e, 0xD, &KD)?,
                0x13 => fading_kind(e, 0x13, &K13)?,
                0x16 => fading_kind(e, 0x16, &K16)?,
                0x10 => boxed_kind(e, Slot::at(0x93A566), true)?,
                0x11 => boxed_kind(e, Slot::at(0x93A56A), false)?,
                _ => false,
            };
            if dies {
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

use crate::marks::plain::{ld, ld80, ldi, st, st80, sti};
use crate::math::x87_float as bh;

/// `box_`'s x87 block as plain Rust on the x87 model (45 instructions), same order and memory places.
unsafe fn box_x87(mk: usize, e: usize, p: usize, m: usize) {
    unsafe {
        let t0 = ld(e, 0x10);
        let t1 = t0 - ld(p, 0x30);
        st(mk, 0x1D07D8, t1);
        let t2 = ld(e, 0x14);
        let t3 = t2 - ld(p, 0x34);
        st(mk, 0x1D07DC, t3);
        let t4 = ld(e, 0x18);
        let t5 = t4 - ld(p, 0x38);
        st(mk, 0x1D07E0, t5);
        let t6 = ld(mk, 0x1D07E0);
        let t7 = ld(mk, 0x1D07DC);
        let t8 = ld(mk, 0x1D07D8);
        let t9 = ld(m, 0);
        let t10 = t9 * t8;
        let t11 = ld(m, 0x10);
        let t12 = t11 * t7;
        let t13 = ld(m, 0x20);
        let t14 = t13 * t6;
        let t15 = t12 + t14;
        let t16 = t10 + t15;
        let t17 = ld(m, 0x4);
        let t18 = t17 * t8;
        let t19 = ld(m, 0x14);
        let t20 = t19 * t7;
        let t21 = ld(m, 0x24);
        let t22 = t21 * t6;
        let t23 = t20 + t22;
        let _t24 = t18 + t23;
        let t25 = ld(m, 0x8);
        let t26 = t8 * t25;
        let t27 = ld(m, 0x18);
        let t28 = t7 * t27;
        let t29 = ld(m, 0x28);
        let t30 = t6 * t29;
        let t31 = t30 + t28;
        let t32 = t26 + t31;
        let t33 = t16.abs();
        sti(mk, 0x1D0740, t33);
        let t34 = t32.abs();
        sti(mk, 0x1D0744, t34);
    }
}

/// `facing`'s x87 block as plain Rust on the x87 model (28 instructions), same order and memory places.
unsafe fn facing_x87(mk: usize, o: usize) {
    unsafe {
        let t0 = ld(mk, 0x1D07D8);
        let t1 = t0 * t0;
        let t2 = ld(mk, 0x1D07DC);
        let t3 = t2 * t2;
        let t4 = ld(mk, 0x1D07E0);
        let t5 = t4 * t4;
        let t6 = t3 + t5;
        let t7 = t1 + t6;
        let t8 = t7.sqrt();
        let t9 = ld(mk, 0x22C) / t8;
        let t10 = ld(mk, 0x1D07D8);
        let t11 = t10 * t9;
        st(mk, 0x1D07D8, t11);
        let t12 = ld(mk, 0x1D07DC);
        let t13 = t12 * t9;
        st(mk, 0x1D07DC, t13);
        let t14 = ld(mk, 0x1D07E0);
        let t15 = t9 * t14;
        st(mk, 0x1D07E0, t15);
        let t16 = ld(mk, 0x1D07D8);
        let t17 = t16 * ld(o, 0x4C);
        let t18 = ld(mk, 0x1D07DC);
        let t19 = t18 * ld(o, 0x50);
        let t20 = ld(mk, 0x1D07E0);
        let t21 = t20 * ld(o, 0x54);
        let t22 = t19 + t21;
        let t23 = t17 + t22;
        st(mk, 0x1D0774, t23);
    }
}

/// `pull_length`'s x87 block as plain Rust on the x87 model (22 instructions), same order and memory places.
unsafe fn pull_length_x87(mk: usize, e: usize, p: usize, l: usize) {
    unsafe {
        let t0 = ld(e, 0x10);
        let t1 = t0 - ld(p, 0x30);
        st(mk, 0x1D07D8, t1);
        let t2 = ld(e, 0x14);
        let t3 = t2 - ld(p, 0x34);
        st(mk, 0x1D07DC, t3);
        let t4 = ld(e, 0x18);
        let t5 = t4 - ld(p, 0x38);
        st(mk, 0x1D07E0, t5);
        let t6 = ld(mk, 0x1D07D8);
        let t7 = t6 * t6;
        let t8 = ld(mk, 0x1D07DC);
        let t9 = t8 * t8;
        let t10 = ld(mk, 0x1D07E0);
        let t11 = t10 * t10;
        let t12 = t9 + t11;
        let t13 = t7 + t12;
        let t14 = t13.sqrt();
        let t15 = t14 - ld(mk, 0x1D55E6);
        st(mk, 0x1D077C, t15);
        st80(l, 0, t14);
    }
}

/// `pull_facing`'s x87 block as plain Rust on the x87 model (32 instructions), same order and memory places.
unsafe fn pull_facing_x87(mk: usize, o: usize, l: usize) {
    unsafe {
        let t0 = bh::from_f80(ld80(l, 0));
        let t1 = ld(mk, 0x22C) / t0;
        let t2 = ld(mk, 0x1D07D8);
        let t3 = t2 * t1;
        st(mk, 0x1D07D8, t3);
        let t4 = ld(mk, 0x1D07DC);
        let t5 = t4 * t1;
        st(mk, 0x1D07DC, t5);
        let t6 = ld(mk, 0x1D07E0);
        let t7 = t1 * t6;
        st(mk, 0x1D07E0, t7);
        let t8 = ldi(o, 0x88);
        let t9 = ld(mk, 0x22C) / t8;
        let t10 = ld(o, 0x4C);
        let t11 = t10 * t9;
        st(mk, 0x1D07E4, t11);
        let t12 = ld(o, 0x50);
        let t13 = t12 * t9;
        st(mk, 0x1D07E8, t13);
        let t14 = ld(o, 0x54);
        let t15 = t9 * t14;
        st(mk, 0x1D07EC, t15);
        let t16 = ld(mk, 0x1D07D8);
        let t17 = t16 * ld(mk, 0x1D07E4);
        let t18 = ld(mk, 0x1D07DC);
        let t19 = t18 * ld(mk, 0x1D07E8);
        let t20 = ld(mk, 0x1D07E0);
        let t21 = t20 * ld(mk, 0x1D07EC);
        let t22 = t19 + t21;
        let t23 = t17 + t22;
        let t24 = -t23;
        st(mk, 0x1D0774, t24);
    }
}

/// `pull_strength`'s x87 block as plain Rust on the x87 model (15 instructions), same order and memory places.
unsafe fn pull_strength_x87(mk: usize, e: usize, o: usize) {
    unsafe {
        let t0 = ldi(o, 0x88);
        let t1 = t0 * ld(mk, 0x1D55EA);
        let t2 = t1 + ld(mk, 0x22C);
        let t3 = ld(mk, 0x22C) / t2;
        let t4 = ld(mk, 0x22C) - t3;
        st(mk, 0x1D0780, t4);
        let t5 = ld(mk, 0x29C);
        let t6 = t5 - ld(e, 0x64);
        let t7 = ld(mk, 0x1D55EE);
        let t8 = t7 * ld(mk, 0x1D0780);
        let t9 = t6 * t8;
        let t10 = t9 * ldi(mk, 0x90);
        let t11 = t10 + ld(e, 0x64);
        sti(mk, 0x1D0780, t11);
        st(e, 0x64, t11);
    }
}

/// `box_place`'s x87 block as plain Rust on the x87 model (36 instructions), same order and memory places.
unsafe fn box_place_x87(mk: usize, e: usize, m: usize) {
    unsafe {
        let t0 = ld(e, 0x18);
        let t1 = ld(e, 0x14);
        let t2 = ld(e, 0x10);
        let t3 = ld(m, 0);
        let t4 = t3 * t2;
        let t5 = ld(m, 0x10);
        let t6 = t5 * t1;
        let t7 = ld(m, 0x20);
        let t8 = t7 * t0;
        let t9 = t6 + t8;
        let t10 = t4 + t9;
        let t11 = ld(m, 0x4);
        let t12 = t11 * t2;
        let t13 = ld(m, 0x14);
        let t14 = t13 * t1;
        let t15 = ld(m, 0x24);
        let t16 = t15 * t0;
        let t17 = t14 + t16;
        let _t18 = t12 + t17;
        let t19 = ld(m, 0x8);
        let t20 = t2 * t19;
        let t21 = ld(m, 0x18);
        let t22 = t1 * t21;
        let t23 = ld(m, 0x28);
        let t24 = t0 * t23;
        let t25 = t24 + t22;
        let t26 = t20 + t25;
        let t27 = t10.abs();
        sti(mk, 0x1D0740, t27);
        let t28 = t26.abs();
        sti(mk, 0x1D0744, t28);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn box_x87(mk: usize, e: usize, p: usize, m: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{e} + 0x10]",
            "fsub dword ptr [{p} + 0x30]",
            "fstp dword ptr [{mk} + 0x1D07D8]",
            "fld dword ptr [{e} + 0x14]",
            "fsub dword ptr [{p} + 0x34]",
            "fstp dword ptr [{mk} + 0x1D07DC]",
            "fld dword ptr [{e} + 0x18]",
            "fsub dword ptr [{p} + 0x38]",
            "fstp dword ptr [{mk} + 0x1D07E0]",
            "fld dword ptr [{mk} + 0x1D07E0]",
            "fld dword ptr [{mk} + 0x1D07DC]",
            "fld dword ptr [{mk} + 0x1D07D8]",
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
            "fistp dword ptr [{mk} + 0x1D0740]",
            ".byte 0xDD, 0xC0",
            ".byte 0xD9, 0xF7",
            ".byte 0xD9, 0xE1",
            "fistp dword ptr [{mk} + 0x1D0744]",
            mk = in(reg) mk,
            e = in(reg) e,
            p = in(reg) p,
            m = in(reg) m,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn facing_x87(mk: usize, o: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1D07D8]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x1D07DC]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x1D07E0]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{mk} + 0x22C]",
            "fld dword ptr [{mk} + 0x1D07D8]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07D8]",
            "fld dword ptr [{mk} + 0x1D07DC]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07DC]",
            "fld dword ptr [{mk} + 0x1D07E0]",
            ".byte 0xDE, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07E0]",
            "fld dword ptr [{mk} + 0x1D07D8]",
            "fmul dword ptr [{o} + 0x4C]",
            "fld dword ptr [{mk} + 0x1D07DC]",
            "fmul dword ptr [{o} + 0x50]",
            "fld dword ptr [{mk} + 0x1D07E0]",
            "fmul dword ptr [{o} + 0x54]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0x1D0774]",
            mk = in(reg) mk,
            o = in(reg) o,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn pull_length_x87(mk: usize, e: usize, p: usize, l: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{e} + 0x10]",
            "fsub dword ptr [{p} + 0x30]",
            "fstp dword ptr [{mk} + 0x1D07D8]",
            "fld dword ptr [{e} + 0x14]",
            "fsub dword ptr [{p} + 0x34]",
            "fstp dword ptr [{mk} + 0x1D07DC]",
            "fld dword ptr [{e} + 0x18]",
            "fsub dword ptr [{p} + 0x38]",
            "fstp dword ptr [{mk} + 0x1D07E0]",
            "fld dword ptr [{mk} + 0x1D07D8]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x1D07DC]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x1D07E0]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{mk} + 0x1D55E6]",
            "fstp dword ptr [{mk} + 0x1D077C]",
            "fstp tbyte ptr [{l}]",
            mk = in(reg) mk,
            e = in(reg) e,
            p = in(reg) p,
            l = in(reg) l,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn pull_facing_x87(mk: usize, o: usize, l: usize) {
        unsafe {
            core::arch::asm!(
                    "fld tbyte ptr [{l}]",
            "fdivr dword ptr [{mk} + 0x22C]",
            "fld dword ptr [{mk} + 0x1D07D8]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07D8]",
            "fld dword ptr [{mk} + 0x1D07DC]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07DC]",
            "fld dword ptr [{mk} + 0x1D07E0]",
            ".byte 0xDE, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07E0]",
            "fild dword ptr [{o} + 0x88]",
            "fdivr dword ptr [{mk} + 0x22C]",
            "fld dword ptr [{o} + 0x4C]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07E4]",
            "fld dword ptr [{o} + 0x50]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07E8]",
            "fld dword ptr [{o} + 0x54]",
            ".byte 0xDE, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07EC]",
            "fld dword ptr [{mk} + 0x1D07D8]",
            "fmul dword ptr [{mk} + 0x1D07E4]",
            "fld dword ptr [{mk} + 0x1D07DC]",
            "fmul dword ptr [{mk} + 0x1D07E8]",
            "fld dword ptr [{mk} + 0x1D07E0]",
            "fmul dword ptr [{mk} + 0x1D07EC]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xE0",
            "fstp dword ptr [{mk} + 0x1D0774]",
            mk = in(reg) mk,
            o = in(reg) o,
            l = in(reg) l,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn pull_strength_x87(mk: usize, e: usize, o: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{o} + 0x88]",
            "fmul dword ptr [{mk} + 0x1D55EA]",
            "fadd dword ptr [{mk} + 0x22C]",
            "fdivr dword ptr [{mk} + 0x22C]",
            "fsubr dword ptr [{mk} + 0x22C]",
            "fstp dword ptr [{mk} + 0x1D0780]",
            "fld dword ptr [{mk} + 0x29C]",
            "fsub dword ptr [{e} + 0x64]",
            "fld dword ptr [{mk} + 0x1D55EE]",
            "fmul dword ptr [{mk} + 0x1D0780]",
            ".byte 0xDE, 0xC9",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x64]",
            "fist dword ptr [{mk} + 0x1D0780]",
            "fstp dword ptr [{e} + 0x64]",
            mk = in(reg) mk,
            e = in(reg) e,
            o = in(reg) o,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn box_place_x87(mk: usize, e: usize, m: usize) {
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
            "fistp dword ptr [{mk} + 0x1D0740]",
            ".byte 0xDD, 0xC0",
            ".byte 0xD9, 0xF7",
            ".byte 0xD9, 0xE1",
            "fistp dword ptr [{mk} + 0x1D0744]",
            mk = in(reg) mk,
            e = in(reg) e,
            m = in(reg) m,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }
}

#[cfg(test)]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::{Op, Spec, differential};

    #[test]
    fn box_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1D0740,
                    hi: 0x1D07E4,
                    reads: &[(0x1D07D8, F32), (0x1D07DC, F32), (0x1D07E0, F32)],
                    writes: &[
                        (0x1D0740, I32),
                        (0x1D0744, I32),
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                    ],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x1C,
                    reads: &[(0x10, F32), (0x14, F32), (0x18, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x30,
                    hi: 0x3C,
                    reads: &[(0x30, F32), (0x34, F32), (0x38, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x2C,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x20, F32),
                        (0x24, F32),
                        (0x28, F32),
                    ],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::box_x87(b[0], b[1], b[2], b[3]) },
            |b| unsafe { super::box_x87(b[0], b[1], b[2], b[3]) },
        );
    }

    #[test]
    fn facing_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x22C,
                    hi: 0x1D07E4,
                    reads: &[
                        (0x22C, F32),
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                    ],
                    writes: &[
                        (0x1D0774, F32),
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                    ],
                },
                Op::Base {
                    lo: 0x4C,
                    hi: 0x58,
                    reads: &[(0x4C, F32), (0x50, F32), (0x54, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::facing_x87(b[0], b[1]) },
            |b| unsafe { super::facing_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn pull_length_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1D077C,
                    hi: 0x1D55EA,
                    reads: &[
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                        (0x1D55E6, F32),
                    ],
                    writes: &[
                        (0x1D077C, F32),
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                    ],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x1C,
                    reads: &[(0x10, F32), (0x14, F32), (0x18, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x30,
                    hi: 0x3C,
                    reads: &[(0x30, F32), (0x34, F32), (0x38, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0xA,
                    reads: &[],
                    writes: &[(0x0, F80)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::pull_length_x87(b[0], b[1], b[2], b[3]) },
            |b| unsafe { super::pull_length_x87(b[0], b[1], b[2], b[3]) },
        );
    }

    #[test]
    fn pull_facing_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x22C,
                    hi: 0x1D07F0,
                    reads: &[
                        (0x22C, F32),
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                        (0x1D07E4, F32),
                        (0x1D07E8, F32),
                        (0x1D07EC, F32),
                    ],
                    writes: &[
                        (0x1D0774, F32),
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                        (0x1D07E4, F32),
                        (0x1D07E8, F32),
                        (0x1D07EC, F32),
                    ],
                },
                Op::Base {
                    lo: 0x4C,
                    hi: 0x8C,
                    reads: &[(0x4C, F32), (0x50, F32), (0x54, F32), (0x88, I32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0xA,
                    reads: &[(0x0, F80)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::pull_facing_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::pull_facing_x87(b[0], b[1], b[2]) },
        );
    }

    #[test]
    fn pull_strength_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x90,
                    hi: 0x1D55F2,
                    reads: &[
                        (0x90, I32),
                        (0x22C, F32),
                        (0x29C, F32),
                        (0x1D0780, F32),
                        (0x1D55EA, F32),
                        (0x1D55EE, F32),
                    ],
                    writes: &[(0x1D0780, F32), (0x1D0780, I32)],
                },
                Op::Base {
                    lo: 0x64,
                    hi: 0x68,
                    reads: &[(0x64, F32)],
                    writes: &[(0x64, F32)],
                },
                Op::Base {
                    lo: 0x88,
                    hi: 0x8C,
                    reads: &[(0x88, I32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::pull_strength_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::pull_strength_x87(b[0], b[1], b[2]) },
        );
    }

    #[test]
    fn box_place_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1D0740,
                    hi: 0x1D0748,
                    reads: &[],
                    writes: &[(0x1D0740, I32), (0x1D0744, I32)],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x1C,
                    reads: &[(0x10, F32), (0x14, F32), (0x18, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x2C,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x20, F32),
                        (0x24, F32),
                        (0x28, F32),
                    ],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::box_place_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::box_place_x87(b[0], b[1], b[2]) },
        );
    }
}
