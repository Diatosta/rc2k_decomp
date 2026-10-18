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
//! Out: the registers as the walk leaves them
//!
//! Caller: `marks::run_frame` (0x4D1640 .., after `sub_4CB64D`). The x87 blocks are the
//! original's instructions over the entries and the game's variables; the fog its Rust
//! version; the ribbon through `part` (1), in the game its Rust version. Refuses, before
//! anything (the walk simulated), null settings, a null entry or link on the way, and a
//! colour index out of its table. Shadow mode runs the original alone (it draws).
//! Self-test: a copy of the whole range (its streak step's return address pointed into
//! the copy) whose ribbon goes to a stand-in, against the Rust version, on a ring of the
//! test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::emitters::{ENTRY_LEN, RING_COUNT_RVA, RING_CURSOR_RVA};
use crate::marks::setup::SETTINGS_RVA;
use crate::math::x87;
use crate::scene::fog_at::fog_at;
use crate::scene::vector3::STATUS_RVA;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::run_ribbons", rva(0x4CB71A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TICKS_RVA: usize = rva(0x764FD0); // g_marksFrameTicks
const STREAK: u32 = 0x12;
const LAST_RVA: usize = rva(0x8EB8E1); // dword_8EB8E1 .. dword_8EB8E9
const PLACE_RVA: usize = rva(0x8EB8D5); // flt_8EB8D5 .. flt_8EB8DD
const SPECULAR_RVA: usize = rva(0x9605B4); // the first vertex's specular
const INDEX_RVA: usize = rva(0x9356C0); // dword_9356C0
const OTHER_RVA: usize = rva(0x765238); // dword_765238
const COLOURS: [(usize, i32); 2] = [(rva(0x939DB2), 257), (rva(0x939BB2), 128)];

/// A call by place: 1 `draw_ribbon`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The original's 0x4CF890 .. 0x4CF94D: a new streak's place turned into its last (+0x1C ..), and a first move.
unsafe fn first_frame(b: u32, e: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{e} + 0x18]",
            "fld dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x10]",
            "fld dword ptr [{b} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{b} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{b} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA2FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{b} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{b} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA300]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{b} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{b} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA304]",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{e} + 0x1C]",
            "fstp dword ptr [{e} + 0x20]",
            "fstp dword ptr [{e} + 0x24]",
            "fld dword ptr [{e} + 0x2C]",
            "fimul dword ptr [{b} + 0x364FD0]",
            "fadd dword ptr [{e} + 0x10]",
            "fstp dword ptr [{e} + 0x10]",
            "fld dword ptr [{e} + 0x30]",
            "fimul dword ptr [{b} + 0x364FD0]",
            "fadd dword ptr [{e} + 0x14]",
            "fstp dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x34]",
            "fimul dword ptr [{b} + 0x364FD0]",
            "fadd dword ptr [{e} + 0x18]",
            "fstp dword ptr [{e} + 0x18]",
            "fld dword ptr [{b} + 0x53A56E]",
            "fimul dword ptr [{b} + 0x364FD0]",
            "fadd dword ptr [{e} + 0x30]",
            "fstp dword ptr [{e} + 0x30]",
            b = in(reg) b,
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4CF94D .. 0x4CF98C: the place moved by the motion, the motion's y by `flt_93A572`.
unsafe fn move_(b: u32, e: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{e} + 0x2C]",
            "fimul dword ptr [{b} + 0x364FD0]",
            "fadd dword ptr [{e} + 0x10]",
            "fstp dword ptr [{e} + 0x10]",
            "fld dword ptr [{e} + 0x30]",
            "fimul dword ptr [{b} + 0x364FD0]",
            "fadd dword ptr [{e} + 0x14]",
            "fstp dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x34]",
            "fimul dword ptr [{b} + 0x364FD0]",
            "fadd dword ptr [{e} + 0x18]",
            "fstp dword ptr [{e} + 0x18]",
            "fld dword ptr [{b} + 0x53A572]",
            "fimul dword ptr [{b} + 0x364FD0]",
            "fadd dword ptr [{e} + 0x30]",
            "fstp dword ptr [{e} + 0x30]",
            b = in(reg) b,
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4CF9A6 .. 0x4CFA2D: the place turned by the view into `flt_8EB8D5` ...
unsafe fn turn_place(b: u32, e: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{e} + 0x18]",
            "fld dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x10]",
            "fld dword ptr [{b} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{b} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{b} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA2FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{b} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{b} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA300]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{b} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{b} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA304]",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{b} + 0x4EB8D5]",
            "fstp dword ptr [{b} + 0x4EB8D9]",
            "fstp dword ptr [{b} + 0x4EB8DD]",
            b = in(reg) b,
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The colour table in use and its size.
unsafe fn colours() -> (usize, i32) {
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

/// The streak step at e (see the header): its eax (not 0: the entry dies).
unsafe fn streak(g: &mut Regs, e: u32, part: &mut Part) -> Result<(), Refused> {
    unsafe {
        let b = game_addr(0) as u32;
        g.eax = game_read(TICKS_RVA);
        let life = get(e + 0x84);
        put(e + 0x84, life.wrapping_sub(g.eax));
        if life < g.eax {
            g.eax = 0xFFFF_FFFF;
            return Ok(());
        }
        g.eax = u32::from(x87::compare_status(get(e + 0x14), get(e + 0x28)));
        if g.eax == 0x100 {
            g.eax = 0xFFFF_FFFF;
            return Ok(());
        }
        if get(e + 0x10) == 0 {
            first_frame(b, e);
        }
        move_on(b, e);
        [g.eax, g.ebx, g.ecx] = [get(e + 0x1C), get(e + 0x20), get(e + 0x24)];
        game_write(LAST_RVA, g.eax);
        game_write(LAST_RVA + 4, g.ebx);
        game_write(LAST_RVA + 8, g.ecx);
        turn_place(b, e);
        let s: u32 = game_read(SETTINGS_RVA);
        g.eax = s;
        if get(s + 0xC) != 0 || get(s + 0x10) == 1 {
            g.eax = game_read(PLACE_RVA);
            g.ebx = game_read(PLACE_RVA + 4);
            g.ecx = game_read(PLACE_RVA + 8);
            put(e + 0x1C, g.eax);
            put(e + 0x20, g.ebx);
            put(e + 0x24, g.ecx);
        }
        game_write(SPECULAR_RVA, fog_at(get(e + 0x24))?);
        g.edx = 0x800;
        let i = x87::int_times_to_int(get(e + 0x84) as i32, get(e + 0x78));
        game_write(INDEX_RVA, i);
        let mut r = Regs {
            eax: 0x20,
            ecx: i as u32,
            esi: game_read::<u32>(colours().0 + 4 * i as usize),
            ..*g
        };
        part(1, &mut r)?;
        *g = Regs {
            esi: e,
            eax: 0,
            ..r
        };
    }
    Ok(())
}

unsafe fn move_on(b: u32, e: u32) {
    unsafe { move_(b, e) }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_ribbons(entry: &Regs, mut part: Part) -> Result<Regs, Refused> {
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
            g.eax = get(e + 8);
            g.ebx = game_read(SETTINGS_RVA);
            if get(g.ebx + 0x60) != 0 {
                if get(e + 0xC) == STREAK {
                    streak(&mut g, e, &mut part)?;
                } else {
                    g.eax = 0;
                }
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
            }
            g.esi = next;
        }
    }
    Ok(g)
}

/// In the game: `draw_ribbon`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::draw_ribbon as ribbon;
    let left = unsafe { ribbon::draw_ribbon(r, &mut ribbon::game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { run_ribbons(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    const LEN: usize = 0x4CFAAF - 0x4CB71A;
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x4CB71A)),
            LEN,
            // mov esi, g_marksRingCursor
            &[0x8B, 0x35, 0x60, 0x56, 0x93, 0x00],
            // The two it takes, and every other call out of the copy (in the functions
            // after it, which the test never runs) moved with it.
            &[
                0x118, 0x323, 0x32A, 0x3D6, 0x5E1, 0x5E8, 0x816, 0x81D, 0x8C9, 0xA2C, 0xA33, 0xBC3,
                0xDC6, 0x1A67, 0x1C42, 0x1ED3, 0x216F, 0x2400, 0x28CE, 0x2C95, 0x2E94, 0x4342,
                0x4389,
            ],
            &|c| (c + 0x40_0000 == 0x4C5E63).then(|| crate::install::stand_ins::stand_in(1)),
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    // The streak step's return address (`push offset loc_4CB752`) into the copy.
    unsafe { copy.patch_u32(0x21, (copy.addr() + 0x38) as u32) };
    const ENTRIES: u32 = 10;
    let mut ring = vec![0u8; ENTRIES as usize * ENTRY_LEN];
    let base = ring.as_mut_ptr() as u32;
    let mut settings = vec![0u32; 0x20];
    let sa = settings.as_mut_ptr() as u32;
    let regions: Vec<Region> = vec![
        at(RING_CURSOR_RVA),
        at(RING_COUNT_RVA),
        at(STATUS_RVA),
        at(INDEX_RVA),
        at(SPECULAR_RVA),
        (game_addr(rva(0x8EB8D5)) as u32, 0x18),
        (base, ring.len()),
    ];
    let mut vars = regions.clone();
    vars.extend(
        [
            SETTINGS_RVA,
            TICKS_RVA,
            OTHER_RVA,
            rva(0x93A56E),
            rva(0x93A572),
            rva(0x63761C),
        ]
        .map(at),
    );
    vars.push((game_addr(rva(0x5EA2CC)) as u32, 0x40));
    let saved = capture(&vars);
    let fl = |v: f32| v.to_bits();
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..4u32 {
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(SETTINGS_RVA, sa);
            put(sa + 0x60, u32::from(case != 3));
            put(sa + 0xC, u32::from(case == 1));
            put(sa + 0x10, case % 2);
            game_write(TICKS_RVA, 2 + case);
            game_write(OTHER_RVA, u32::from(case == 2));
            game_write(rva(0x93A56E), fl(-0.5));
            game_write(rva(0x93A572), fl(-0.25));
            game_write(rva(0x63761C), 0x40u32);
            for n in 0..16 {
                game_write(
                    rva(0x5EA2CC) + 4 * n,
                    fl(if n % 5 == 0 { 1.0 } else { 0.0 }),
                );
            }
            game_write(rva(0x5EA2CC) + 0x38, fl(30.0));
            std::ptr::write_bytes(base as *mut u8, 0, ring.len());
            for n in 0..ENTRIES {
                let e = base + n * ENTRY_LEN as u32;
                let at = |k: u32| base + ((k + ENTRIES) % ENTRIES) * ENTRY_LEN as u32;
                put(e, at(n + ENTRIES - 1));
                put(e + 4, at(n + 1));
                put(e + 0xC, if n % 3 == 0 { 5 } else { STREAK });
                put(
                    e + 0x10,
                    if n % 4 == 1 {
                        0
                    } else {
                        fl(n as f32 * 2.0 - 7.0)
                    },
                );
                put(e + 0x14, fl(4.0 + n as f32));
                put(e + 0x18, fl(n as f32 * 0.5));
                put(e + 0x28, if n == 5 { fl(99.0) } else { fl(-1.0) });
                put(e + 0x2C, fl(0.25));
                put(e + 0x30, fl(-0.125 * n as f32));
                put(e + 0x34, fl(0.5));
                put(e + 0x78, fl(1.0 / 64.0));
                put(e + 0x84, if n == 7 { 1 } else { 300 + 40 * n });
            }
            game_write(RING_CURSOR_RVA, base + 2 * ENTRY_LEN as u32);
            game_write(RING_COUNT_RVA, 7u32);
        };
        prepare();
        if unsafe { refusal() }.is_some() {
            t.check(true, String::new);
            continue;
        }
        take_called();
        let mut ro = Regs::SENTINEL;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            run_ribbons(&Regs::SENTINEL, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            take_called(),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!(
                "case {case}: original {:x?} {} calls; rust {:x?} {} calls; state first differs at {:?}",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len(),
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b)
            )
        });
    }
    restore(&vars, &saved);
    drop((ring, settings));
    t.finish()
}
