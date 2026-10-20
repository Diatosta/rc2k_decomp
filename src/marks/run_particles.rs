//! `marks::run_particles` (0x4CFB49, with its chunk 0x4D02B4 .. 0x4D05BD): the marks'
//! particle ring's frame. Nothing when the settings' +4 is 0; else `marks::set_quad_uvs`,
//! then `g_marksRingUsed` (as it is at the start) entries from `g_marksRingFree` along +4:
//! - the life (+0x44) less `g_marksFrameTicks`; the entry dies when that borrows, or when
//!   its y (+0x10) is below its ground (+0x24) (`fcomp`, `g_fpuStatus` 0x100), or when
//!   its kind's step gives 1;
//! - the kinds' steps (each the place moved by the motion times the ticks): 0 (dust:
//!   +0x2C gains `flt_93C680` a tick, +0x3C `dword_93C684`; at or above `flt_93C688`
//!   `marks::burst_particles` and death, else `marks::trail_particles` while it rises,
//!   the last place kept), 4 (trail: `flt_93C68C`, `dword_93C690`; drawn with colour
//!   `dword_922B38`[life times +0x40]), 8 and 9 (burst: +0x10 also moved by +0x30, which
//!   gains `flt_93C694` / `flt_93C6A8`, the motion damped by `flt_93C698` .. / `flt_93C6AC`
//!   .., colour `dword_93A618`[+0x48 + life times +0x40], none: death; 9 one time in 128
//!   drawn 0x386A0 bigger), each drawn by `marks::draw_particle`;
//! - a dead entry: `g_marksRingUsed` less 1, and the entry moved to just before
//!   `g_marksRingFree` (or that moved on past it when it is the entry).
//!
//! Every eighth marks step (`dword_76513C`): `dword_5165E4` 1 and a drift puff near the
//! view (`flt_5EA3CC` + 0x30) from two steps of the marks' random by
//! `marks::add_drift_particle`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as the path leaves them
//!
//! Caller: `marks::run_frame` (0x4D165E). Its calls through `part` (1 `set_quad_uvs`, 2
//! `burst_particles`, 3 `trail_particles`, 4 `draw_particle` with three values, 5
//! `add_drift_particle` with six); in the game their Rust versions. Refuses, before
//! anything, null settings, a null entry or link on the way, a kind other than 0, 4, 8,
//! 9 (the original's stack goes wrong: section 16) and a colour index out of its table;
//! mid-way only its callees' own refusals (a broken ring).
//! Its first instruction is a short jump: private copy of the whole range (the five
//! functions between its chunks taken along, never run from it; its seven calls
//! relocated), manual patch. Shadow mode runs the original alone (it draws). Self-test: a
//! copy (its handlers' return address pointed into the copy) whose calls go to stand-ins
//! (the x87 ones taking their values off), against the Rust version on the same, on a
//! ring of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::blocks::{RING_ENTRY, RING_FREE_RVA, RING_USED_RVA};
use crate::marks::setup::{SEED_RVA, SETTINGS_RVA};
use crate::math::x87;
use crate::scene::matrix3::extended;
use crate::scene::vector3::STATUS_RVA;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("marks::run_particles", rva(0x4CFB49), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0xA74;
const CALLS: [usize; 10] = [
    0x33, 0x21B, 0x221, 0x408, 0x59A, 0x7E2, 0x7F6, 0x89E, 0x975, 0xA63,
];
const TICKS_RVA: usize = rva(0x764FD0); // g_marksFrameTicks
const STEPS_RVA: usize = rva(0x76513C); // dword_76513C
const PUFF_FLAG_RVA: usize = rva(0x5165E4); // dword_5165E4
const VIEW_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC: the view's place at +0x30
const INDEX_RVA: usize = rva(0x93A5E8); // dword_93A5E8
const TRAIL_COLOURS_RVA: usize = rva(0x922B38); // dword_922B38: 298 dwords
const TRAIL_COLOURS: i32 = 298;
const BURST_COLOURS_RVA: usize = rva(0x93A618); // dword_93A618: 0x2000 bytes
const BURST_COLOURS_LEN: i32 = 0x2000;
const UV_K_RVA: usize = rva(0x8EB969); // flt_8EB969
const DUST: [usize; 3] = [rva(0x93C680), rva(0x93C684), rva(0x93C688)];
const TRAIL: [usize; 2] = [rva(0x93C68C), rva(0x93C690)];
/// Kind 8's and 9's: the +0x30 gain, the three dampings, the +0x3C gain.
const BURST: [[usize; 5]; 2] = [
    [
        rva(0x93C694),
        rva(0x93C698),
        rva(0x93C69C),
        rva(0x93C6A0),
        rva(0x93C6A4),
    ],
    [
        rva(0x93C6A8),
        rva(0x93C6AC),
        rva(0x93C6B0),
        rva(0x93C6B4),
        rva(0x93C6B8),
    ],
];
/// The puff's: x wave (three factors, size), z wave (the same), y (add, factor, add),
/// the motion's two factors.
const PUFF_X: [usize; 4] = [rva(0x93C618), rva(0x93C61C), rva(0x93C620), 0];
const PUFF_Z: [usize; 4] = [rva(0x93C624), rva(0x93C628), rva(0x93C62C), 0];
const PUFF_Y: [usize; 3] = [rva(0x93C630), rva(0x93C634), rva(0x93C638)];
const PUFF_M: [usize; 2] = [rva(0x93C63C), rva(0x93C640)];
const PUFF_RVA: usize = rva(0x93A600); // flt_93A600 .. flt_93A614
const BIGGER: u32 = 0x386A0;

/// A call by place (see the header), with the x87 values it takes (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// A kind's colour index (life times +0x40, `fistp`) and whether it is in its table.
unsafe fn colour_index(e: u32, life: u32) -> (i32, bool) {
    unsafe {
        let i = x87::int_times_to_int(life as i32, get(e + 0x40));
        let ok = match get(e + 8) {
            4 => (0..TRAIL_COLOURS).contains(&i),
            _ => {
                let off = (get(e + 0x48) as i32).wrapping_add(i.wrapping_mul(4));
                i >= 0 && (0..=BURST_COLOURS_LEN - 4).contains(&off)
            }
        };
        (i, ok)
    }
}

/// Why it refuses, if it does: the walk as the original makes it, nothing written.
unsafe fn refusal() -> Option<&'static str> {
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        if settings == 0 {
            return Some("no marks settings");
        }
        if get(settings + 4) == 0 {
            return None;
        }
        let ticks: u32 = game_read(TICKS_RVA);
        let mut e: u32 = game_read(RING_FREE_RVA);
        let used = game_read::<u32>(RING_USED_RVA) as i32;
        if used > 0 && (e == 0 || get(e) == 0) {
            return Some("a null particle ring entry");
        }
        for _ in 0..used.max(0) {
            if e == 0 || get(e) == 0 || get(e + 4) == 0 {
                return Some("a null particle ring entry or link");
            }
            let life = get(e + 0x44);
            let kind = get(e + 8);
            if life >= ticks && x87::compare_status(get(e + 0x10), get(e + 0x24)) != 0x100 {
                if ![0, 4, 8, 9].contains(&kind) {
                    return Some("a particle of another kind (the original's stack goes wrong)");
                }
                if kind != 0 && !colour_index(e, life - ticks).1 {
                    return Some("a particle colour index out of its table");
                }
            }
            e = get(e + 4);
        }
        None
    }
}

/// The step of the kind at e (see the header): its eax (1: the entry dies).
unsafe fn step(g: &mut Regs, e: u32, part: &mut Part) -> Result<(), Refused> {
    unsafe {
        let t: i32 = game_read(TICKS_RVA);
        let move_place = |with_up: bool| {
            put(e + 0xC, x87::plus_times_int(get(e + 0xC), get(e + 0x28), t));
            let y = if with_up {
                x87::plus_sum_times_int(get(e + 0x10), get(e + 0x2C), get(e + 0x30), t)
            } else {
                x87::plus_times_int(get(e + 0x10), get(e + 0x2C), t)
            };
            put(e + 0x10, y);
            put(
                e + 0x14,
                x87::plus_times_int(get(e + 0x14), get(e + 0x34), t),
            );
        };
        let gain_int = |k: usize| {
            let v = x87::int_times_int_plus_int(game_read(k), t, get(e + 0x3C) as i32);
            put(e + 0x3C, v as u32);
        };
        let keep_last = |g: &mut Regs| {
            [g.eax, g.ebx, g.ecx] = [get(e + 0xC), get(e + 0x10), get(e + 0x14)];
            put(e + 0x18, g.eax);
            put(e + 0x1C, g.ebx);
            put(e + 0x20, g.ecx);
        };
        let place = || {
            [
                extended(get(e + 0xC)),
                extended(get(e + 0x10)),
                extended(get(e + 0x14)),
            ]
        };
        match get(e + 8) {
            0 => {
                move_place(false);
                put(
                    e + 0x2C,
                    x87::plus_times_int(get(e + 0x2C), game_read(DUST[0]), t),
                );
                gain_int(DUST[1]);
                let s = x87::compare_status(game_read(DUST[2]), get(e + 0x2C));
                game_write(STATUS_RVA, s as u32);
                if s != 0x100 {
                    part(2, g, &[])?;
                    g.eax = 1;
                    return Ok(());
                }
                if get(e + 0x2C) & 0x8000_0000 == 0 {
                    part(3, g, &[])?;
                }
                keep_last(g);
                g.eax = 0;
            }
            4 => {
                move_place(false);
                put(
                    e + 0x2C,
                    x87::plus_times_int(get(e + 0x2C), game_read(TRAIL[0]), t),
                );
                gain_int(TRAIL[1]);
                let (i, _) = colour_index(e, get(e + 0x44));
                game_write(INDEX_RVA, i);
                g.eax = get(e + 0x3C) & 0xFF00_0000;
                g.ecx = (game_read::<u32>(TRAIL_COLOURS_RVA + 4 * i as usize) & 0xFF_FFFF) | g.eax;
                g.edx = get(e + 0x38);
                part(4, g, &place())?;
                keep_last(g);
                g.eax = 0;
            }
            kind => {
                let k = BURST[(kind == 9) as usize];
                move_place(true);
                put(
                    e + 0x30,
                    x87::plus_times_int(get(e + 0x30), game_read(k[0]), t),
                );
                put(
                    e + 0x28,
                    x87::less_own_times(get(e + 0x28), game_read(k[1])),
                );
                put(
                    e + 0x2C,
                    x87::less_own_times(get(e + 0x2C), game_read(k[2])),
                );
                put(
                    e + 0x34,
                    x87::less_own_times(get(e + 0x34), game_read(k[3])),
                );
                gain_int(k[4]);
                let (i, _) = colour_index(e, get(e + 0x44));
                game_write(INDEX_RVA, i);
                g.ebx = get(e + 0x48);
                g.eax = game_read::<u32>(
                    BURST_COLOURS_RVA + (g.ebx as usize).wrapping_add(4 * i as usize),
                );
                g.ecx = get(e + 0x3C) & 0xFF00_0000;
                g.eax &= 0xFF_FFFF;
                if g.eax == 0 {
                    g.eax = 1;
                    return Ok(());
                }
                g.ecx |= g.eax;
                g.edx = get(e + 0x38);
                if kind == 9 {
                    let s = game_read::<u32>(SEED_RVA)
                        .wrapping_mul(0x9D)
                        .wrapping_add(0x47);
                    game_write(SEED_RVA, s);
                    g.ebx = s.rotate_right(16) & 0x7F;
                    if g.ebx == 0 {
                        g.edx = g.edx.wrapping_add(BIGGER);
                    }
                }
                part(4, g, &place())?;
                g.eax = 0;
            }
        }
    }
    Ok(())
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_particles(entry: &Regs, mut part: Part) -> Result<Regs, Refused> {
    if let Some(why) = unsafe { refusal() } {
        return Err(BadInput(why).into());
    }
    let mut g = *entry;
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        g.esi = settings;
        if get(settings + 4) == 0 {
            return Ok(g);
        }
        part(1, &mut g, &[])?;
        g.esi = game_read(RING_FREE_RVA);
        let mut counter = game_read::<u32>(RING_USED_RVA) as i32;
        loop {
            counter = counter.wrapping_sub(1);
            if counter < 0 {
                break;
            }
            let e = g.esi;
            let next = get(e + 4);
            g.eax = game_read(TICKS_RVA);
            let life = get(e + 0x44);
            put(e + 0x44, life.wrapping_sub(g.eax));
            let mut dies = life < g.eax;
            if !dies {
                let s = x87::compare_status(get(e + 0x10), get(e + 0x24));
                game_write(STATUS_RVA, s as u32);
                dies = s == 0x100;
                if !dies {
                    step(&mut g, e, &mut part)?;
                    dies = g.eax == 1;
                }
            }
            if dies {
                game_write(
                    RING_USED_RVA,
                    game_read::<u32>(RING_USED_RVA).wrapping_sub(1),
                );
                if e == game_read::<u32>(RING_FREE_RVA) {
                    g.eax = get(e + 4);
                    game_write(RING_FREE_RVA, g.eax);
                } else {
                    g.ebx = get(e);
                    g.ecx = get(e + 4);
                    put(g.ebx + 4, g.ecx);
                    put(g.ecx, g.ebx);
                    g.ebx = game_read(RING_FREE_RVA);
                    g.ecx = get(g.ebx);
                    put(g.ebx, e);
                    put(g.ecx + 4, e);
                    put(e, g.ecx);
                    put(e + 4, g.ebx);
                }
            }
            g.esi = next;
        }
        if game_read::<u32>(STEPS_RVA) & 7 != 0 {
            return Ok(g);
        }
        game_write(PUFF_FLAG_RVA, 1u32);
        let seed = || {
            let s = game_read::<u32>(SEED_RVA)
                .wrapping_mul(0x9D)
                .wrapping_add(0x47);
            game_write(SEED_RVA, s);
            s & 0xFFFF
        };
        let r = seed();
        g.ebp = game_addr(VIEW_RVA) as u32;
        let view = |o: u32| get(g.ebp + o);
        let uv_k: u32 = game_read(UV_K_RVA);
        let wave = |k: [usize; 4], add: u32, cosine: bool| {
            x87::int_times3_wave_plus(
                r as i32,
                [uv_k, game_read(k[0]), game_read(k[1])],
                game_read(k[2]),
                add,
                cosine,
            )
        };
        let x = wave(PUFF_X, view(0x30), false);
        let y = x87::loaded(view(0x34));
        let z = wave(PUFF_Z, view(0x38), true);
        g.eax = seed();
        let up = x87::int_times_plus_times_plus(
            g.eax as i32,
            uv_k,
            game_read(PUFF_Y[0]),
            game_read(PUFF_Y[1]),
            game_read(PUFF_Y[2]),
        );
        let mx = x87::sub_mul(view(0x30), x, game_read(PUFF_M[0]));
        let mz = x87::sub_mul(view(0x38), z, game_read(PUFF_M[1]));
        for (n, v) in [x, y, z, mx, up, mz].into_iter().enumerate() {
            game_write(PUFF_RVA + 4 * n, v);
        }
        g.eax = 0;
        let six = [mz, up, mx, z, y, x].map(extended);
        part(5, &mut g, &six)?;
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::marks::{
        add_drift_particle, burst_particles, draw_particle, set_quad_uvs, trail_particles,
    };
    unsafe {
        let left = match place {
            1 => set_quad_uvs::set_quad_uvs(r),
            2 => burst_particles::burst_particles(r)?,
            3 => trail_particles::trail_particles(r)?,
            4 => {
                draw_particle::draw_particle(r, [v[0], v[1], v[2]], &mut draw_particle::game_part)?
            }
            _ => {
                add_drift_particle::add_drift_particle(
                    r.eax,
                    [v[0], v[1], v[2], v[3], v[4], v[5]],
                )?;
                return Ok(());
            }
        };
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        };
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { run_particles(r, &mut game_part) } {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// A call as the test notes it: place, eax .. esi, the x87 values.
type Noted = (u32, [u32; 5], Vec<[u8; 10]>);

static NOTED: Mutex<Vec<Noted>> = Mutex::new(Vec::new());

fn note(place: u32, r: &Regs, v: &[[u8; 10]]) {
    NOTED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.edx, r.esi], v.to_vec()));
}

fn note_regs(place: u32, r: &mut Regs) {
    note(place, r, &[]);
}

extern "C" fn note_x87(place: u32, r: *const Regs, v: *const [u8; 10], n: u32) {
    let vals = unsafe { std::slice::from_raw_parts(v, n as usize) };
    note(place, unsafe { &*r }, vals);
}

/// The test's stand-in for `draw_particle`: its three values taken off and noted.
#[unsafe(naked)]
unsafe extern "C" fn draw_stand_in() {
    core::arch::naked_asm!(
        "lea esp, [esp - 30]",
        "fstp tbyte ptr [esp]",
        "fstp tbyte ptr [esp + 10]",
        "fstp tbyte ptr [esp + 20]",
        "pushfd",
        "pushad",
        "mov eax, esp",
        "lea ecx, [esp + 36]",
        "push 3",
        "push ecx",
        "push eax",
        "push 4",
        "call {note}",
        "add esp, 16",
        "popad",
        "popfd",
        "lea esp, [esp + 30]",
        "ret",
        note = sym note_x87,
    )
}

/// The test's stand-in for `add_drift_particle`: its six values taken off and noted.
#[unsafe(naked)]
unsafe extern "C" fn drift_stand_in() {
    core::arch::naked_asm!(
        "lea esp, [esp - 60]",
        "fstp tbyte ptr [esp]",
        "fstp tbyte ptr [esp + 10]",
        "fstp tbyte ptr [esp + 20]",
        "fstp tbyte ptr [esp + 30]",
        "fstp tbyte ptr [esp + 40]",
        "fstp tbyte ptr [esp + 50]",
        "pushfd",
        "pushad",
        "mov eax, esp",
        "lea ecx, [esp + 36]",
        "push 6",
        "push ecx",
        "push eax",
        "push 5",
        "call {note}",
        "add esp, 16",
        "popad",
        "popfd",
        "lea esp, [esp + 60]",
        "ret",
        note = sym note_x87,
    )
}

fn self_test() -> (usize, usize) {
    use crate::install::stand_ins::{set_handler, stand_in};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    set_handler(note_regs);
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::with_bytes(
            game_addr(rva(0x4CFB49)),
            LEN,
            &[
                // jmp short +0x21, test byte_4F6B86, 0FFh
                (0, &[0xEB, 0x21, 0xF6, 0x05, 0x86, 0x6B, 0x4F, 0x00, 0xFF]),
                // 0x4CFD6A, a hooked function's start: call 0x44E3D1
                (0x221, &[0xE8, 0x62, 0xE6, 0xF7, 0xFF]),
            ],
            &CALLS,
            &|c| match c + 0x40_0000 {
                0x4CFD6A => Some(stand_in(1)),
                0x4D0146 => Some(stand_in(2)),
                0x4CFFE3 => Some(stand_in(3)),
                0x4CFDD5 => Some(draw_stand_in as *const () as usize),
                0x4CFF57 => Some(drift_stand_in as *const () as usize),
                _ => None,
            },
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    // The kinds' return address (`push offset loc_4CFC25`) into the copy.
    unsafe { copy.patch_u32(0xB0, (copy.addr() + 0xDC) as u32) };
    const ENTRIES: u32 = 12;
    let mut ring = vec![0u8; (ENTRIES * RING_ENTRY) as usize];
    let base = ring.as_mut_ptr() as u32;
    let mut settings = vec![0u32; 4];
    let sa = settings.as_mut_ptr() as u32;
    let regions: Vec<Region> = [
        RING_FREE_RVA,
        RING_USED_RVA,
        SEED_RVA,
        STATUS_RVA,
        INDEX_RVA,
        PUFF_FLAG_RVA,
    ]
    .map(at)
    .into_iter()
    .chain([(game_addr(PUFF_RVA) as u32, 24), (base, ring.len())])
    .collect();
    let mut vars = regions.clone();
    vars.extend([SETTINGS_RVA, TICKS_RVA, STEPS_RVA].map(at));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..5u32 {
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(SETTINGS_RVA, sa);
            put(sa + 4, u32::from(case != 4));
            game_write(TICKS_RVA, 3 + case);
            game_write(STEPS_RVA, if case % 2 == 0 { 8 } else { 3 });
            game_write(SEED_RVA, case.wrapping_mul(0x2545_F491) | 1);
            std::ptr::write_bytes(base as *mut u8, 0, ring.len());
            for n in 0..ENTRIES {
                let e = base + n * RING_ENTRY;
                let at = |k: u32| base + ((k + ENTRIES) % ENTRIES) * RING_ENTRY;
                put(e, at(n + ENTRIES - 1));
                put(e + 4, at(n + 1));
                let k = (n + case) as f32;
                put(e + 8, [0, 4, 8, 9][((n + case) % 4) as usize]);
                for o in 0..9u32 {
                    put(e + 0xC + 4 * o, (k * 0.5 - o as f32 * 1.25).to_bits());
                }
                // y above its ground but for one; motion up and down.
                put(e + 0x10, (5.0 + k).to_bits());
                put(e + 0x24, if n == 5 { 99.0f32 } else { -2.0 }.to_bits());
                put(
                    e + 0x2C,
                    (if n % 3 == 0 { 0.5f32 } else { -0.25 }).to_bits(),
                );
                put(e + 0x30, 0.125f32.to_bits());
                put(e + 0x38, 0x4000 + n);
                put(e + 0x3C, 0x7F00_0000 + n * 0x10_0000);
                put(e + 0x40, (1.0f32 / 64.0).to_bits());
                put(e + 0x44, if n == 3 { 1 } else { 400 + n * 50 });
                put(e + 0x48, (n % 8) * 0x400);
            }
            game_write(RING_FREE_RVA, base + 2 * RING_ENTRY);
            game_write(RING_USED_RVA, 8u32);
        };
        if unsafe {
            prepare();
            refusal()
        }
        .is_some()
        {
            t.check(true, String::new);
            continue;
        }
        prepare();
        NOTED.lock().unwrap().clear();
        let mut ro = Regs::SENTINEL;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (
            Some(regs(&ro)),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&regions),
        );
        prepare();
        let mine = unsafe {
            run_particles(&Regs::SENTINEL, &mut |p, r, v| {
                note(p, r, v);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!(
                "case {case}: original {:x?} {} calls; rust {:x?} {} calls; calls same {}; state first differs at {:?}",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len(),
                theirs.1 == mine.1,
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b)
            )
        });
    }
    restore(&vars, &saved);
    drop((ring, settings));
    t.finish()
}
