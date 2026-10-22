//! `marks::run_rain` (0x4C16F9): the rain's frame. `dword_8BC50D` the settings' +0x2764
//! (the drops; none: `dword_8BC405` 0 and back), held to 0x100; the settings' +0x28 ..
//! +0x48 into `dword_8BC441` .., +0x1C .. +0x24 into `dword_8BC435` ..; `marks::place_view`,
//! `scene::depth_buffer_off`, render states 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 0, 0x1B = 1,
//! `scene::clear_texture`; the four vertices' rhw `dword_8C5F91` (through `dword_62400C`),
//! colours 0xFFFFFF, 0, 0x28000000, 0x28FFFFFF, the rest 0, `g_polygonVertexCount` 4;
//! `marks::drift_offsets` when `dword_7652A0` is set; the view's entry (the settings' +8,
//! 12 bytes at `flt_8BC489` and `dword_8BC4C5`): when the settings' +0xC or
//! `dword_7652A0` is set its move since the last frame scaled; the move (`dword_8BC47D`
//! ..) held to `flt_8C5FA5` long when longer than `flt_8C5FA1`; the view's place kept
//! (y plus `dword_8BC505`); `dword_8BC505` the clock's low 20 bits times `flt_8C5FB5`
//! when drifting. Then each drop k from the last: its noise place (`g_marksNoiseA` .. [k]
//! plus the view and the drift, wrapped to 0x40000 and centred) as whole numbers to
//! `marks::draw_streak`, and when the settings' +0x10 is set its turned place kept
//! (`dword_8BD111` .. [k], `fistp`). `dword_8BC405` 1.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as the path leaves them
//!
//! Caller: `marks::run_frame` (0x4D1569). Its calls through `part` (1 `place_view`, 2
//! `depth_buffer_off`, 3 a render state, 4 `clear_texture`, 5 `drift_offsets`, 6
//! `draw_streak` with three values); in the game their Rust versions (`clear_texture`, the
//! platform's, through the game: section 15). The x87 blocks are the original's
//! instructions over the game's variables. Refuses null settings and a view entry past
//! the five (the settings' +8), before anything. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose calls go to stand-ins (the streak's taking its values
//! off), against the Rust version, on the game's tables with settings of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::setup::SETTINGS_RVA;
use crate::math::{x87, x87_run};
use crate::scene::vector3::STATUS_RVA;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::run_rain", rva(0x4C16F9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DROPS_RVA: usize = rva(0x8BC50D); // dword_8BC50D
const DRAWN_RVA: usize = rva(0x8BC405); // dword_8BC405
const MOST_DROPS: i32 = 0x100;
const COPIES_A: usize = rva(0x8BC441); // dword_8BC441 .. : the settings' +0x28 ..
const COPIES_B: usize = rva(0x8BC435); // dword_8BC435 .. : the settings' +0x1C ..
const RHW_RVA: usize = rva(0x8C5F91); // dword_8C5F91
const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const DRIFTING_RVA: usize = rva(0x7652A0); // dword_7652A0
const ENTRIES: u32 = 5;
const ENTRY_RVA: usize = rva(0x8BC489); // flt_8BC489: 5 of 12 bytes
const LAST_RVA: usize = rva(0x8BC4C5); // dword_8BC4C5: 5 of 12 bytes
const MOVE_RVA: usize = rva(0x8BC47D); // dword_8BC47D .. dword_8BC485
const VIEW_RVA: usize = rva(0x8BDD7D); // dword_8BDD7D .. dword_8BDD85
const DRIFT_RVA: usize = rva(0x8BC505); // dword_8BC505
const DRIFT_K_RVA: usize = rva(0x8C5FB5); // flt_8C5FB5
const CLOCK_RVA: usize = rva(0x613830); // g_sceneClock
const NOISE_RVA: [usize; 3] = [rva(0x8BC511), rva(0x8BC911), rva(0x8BCD11)];
const SPREAD_RVA: usize = rva(0x8BC429); // dword_8BC429
const TURNED_RVA: usize = rva(0x8BDE3D); // flt_8BDE3D .. flt_8BDE45
const KEPT_RVA: [usize; 3] = [rva(0x8BD111), rva(0x8BD511), rva(0x8BD911)];
const CLEAR_TEXTURE_RVA: usize = rva(0x44E958); // scene::clear_texture
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];
const COLOURS: [u32; 4] = [0xFF_FFFF, 0, 0x2800_0000, 0x28FF_FFFF];

/// A call by place (see the header), with the x87 values it takes (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The original's 0x4C18E7 .. 0x4C1935: the view's move since last frame, scaled, into its entry.
unsafe fn box_moved(b: u32, e: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{b} + 0x4BDD7D]",
            "fisub dword ptr [{b} + {e} + 0x4BC4C5]",
            "fmul dword ptr [{b} + 0x4C5F95]",
            "fstp dword ptr [{b} + {e} + 0x4BC489]",
            "fild dword ptr [{b} + 0x4BDD81]",
            "fiadd dword ptr [{b} + 0x4BC505]",
            "fisub dword ptr [{b} + {e} + 0x4BC4C9]",
            "fmul dword ptr [{b} + 0x4C5F99]",
            "fstp dword ptr [{b} + {e} + 0x4BC48D]",
            "fild dword ptr [{b} + 0x4BDD85]",
            "fisub dword ptr [{b} + {e} + 0x4BC4CD]",
            "fmul dword ptr [{b} + 0x4C5F9D]",
            "fstp dword ptr [{b} + {e} + 0x4BC491]",
            b = in(reg) b,
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C1965 .. 0x4C1996: the move's length into `flt_8BDD5D`, compared with `flt_8C5FA1` (`fstsw` into `g_fpuStatus`).
unsafe fn move_length(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x4BC47D]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x4BC481]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x4BC485]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fst dword ptr [{b} + 0x4BDD5D]",
            "fcomp dword ptr [{b} + 0x4C5FA1]",
            "fstsw word ptr [{b} + 0x31C814]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C19AC .. 0x4C19F2: the move scaled to `flt_8C5FA5` over its length, times its three factors.
unsafe fn move_held(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x4BDD5D]",
            "fdivr dword ptr [{b} + 0x4C5FA5]",
            ".byte 0xD9, 0xC0",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x4BC47D]",
            "fmul dword ptr [{b} + 0x4C5FA9]",
            "fstp dword ptr [{b} + 0x4BC47D]",
            "fmul dword ptr [{b} + 0x4BC481]",
            "fmul dword ptr [{b} + 0x4C5FAD]",
            "fstp dword ptr [{b} + 0x4BC481]",
            "fmul dword ptr [{b} + 0x4BC485]",
            "fmul dword ptr [{b} + 0x4C5FB1]",
            "fstp dword ptr [{b} + 0x4BC485]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C1A0A .. 0x4C1A1C: the view's y plus the drift (`fistp`) kept in its entry.
unsafe fn keep_y(b: u32, e: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{b} + 0x4BDD81]",
            "fiadd dword ptr [{b} + 0x4BC505]",
            "fistp dword ptr [{b} + {e} + 0x4BC4C9]",
            b = in(reg) b,
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_rain(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        if get(s + 8) >= ENTRIES {
            return Err(BadInput("a rain view entry past the five").into());
        }
        let b = game_addr(0) as u32;
        g.esi = s;
        let mut drops = get(s + 0x2764) as i32;
        game_write(DROPS_RVA, drops);
        if drops == 0 {
            game_write(DRAWN_RVA, 0u32);
            return Ok(g);
        }
        if drops > MOST_DROPS {
            drops = MOST_DROPS;
            game_write(DROPS_RVA, drops);
        }
        for n in 0..9 {
            game_write(COPIES_A + 4 * n, get(s + 0x28 + 4 * n as u32));
        }
        for n in 0..3 {
            game_write(COPIES_B + 4 * n, get(s + 0x1C + 4 * n as u32));
        }
        part(1, &mut g, &[])?;
        part(2, &mut g, &[])?;
        for (value, state) in STATES {
            g.eax = value;
            g.ebx = state;
            part(3, &mut g, &[])?;
        }
        part(4, &mut g, &[])?;
        g.edi = game_addr(VERTICES_RVA) as u32;
        g.eax = game_read(RHW_RVA);
        for (n, colour) in COLOURS.into_iter().enumerate() {
            let v = VERTICES_RVA + 0x20 * n;
            game_write(SCRATCH_RVA, g.eax);
            game_write(v + 0xC, x87::int_to_float(g.eax as i32));
            game_write(v + 0x10, colour);
            for o in [0x14, 0x18, 0x1C] {
                game_write(v + o, 0u32);
            }
        }
        game_write(COUNT_RVA, 4u32);
        let drifting = game_read::<u32>(DRIFTING_RVA) != 0;
        if drifting {
            part(5, &mut g, &[])?;
        }
        let s: u32 = game_read(SETTINGS_RVA);
        g.ebp = s;
        let e = get(s + 8) * 12;
        if get(s + 0xC) != 0 || game_read::<u32>(DRIFTING_RVA) != 0 {
            box_moved(b, e);
        }
        g.ebp = e;
        for n in 0..3 {
            game_write(
                MOVE_RVA + 4 * n,
                game_read::<u32>(ENTRY_RVA + e as usize + 4 * n),
            );
        }
        move_length(b);
        let status = game_read::<u32>(STATUS_RVA) & 0x700;
        game_write(STATUS_RVA, status);
        if status != 0x100 {
            move_held(b);
        }
        game_write(LAST_RVA + e as usize, game_read::<u32>(VIEW_RVA));
        keep_y(b, e);
        game_write(LAST_RVA + e as usize + 8, game_read::<u32>(VIEW_RVA + 8));
        if game_read::<u32>(DRIFTING_RVA) != 0 {
            g.eax = game_read::<u32>(CLOCK_RVA) & 0xF_FFFF;
            game_write(
                DRIFT_RVA,
                x87::int_times_to_int(g.eax as i32, game_read(DRIFT_K_RVA)),
            );
        }
        for k in (0..drops.max(0) as usize).rev() {
            g.esi = k as u32;
            let drift: u32 = game_read(DRIFT_RVA);
            let spread: u32 = game_read(SPREAD_RVA);
            let view = |n: usize| game_read::<u32>(VIEW_RVA + 4 * n);
            g.eax = game_read::<u32>(NOISE_RVA[0] + 4 * k);
            g.ebx = game_read::<u32>(NOISE_RVA[1] + 4 * k);
            g.ecx = game_read::<u32>(NOISE_RVA[2] + 4 * k);
            g.ebx = g.ebx.wrapping_add(drift);
            g.eax = g.eax.wrapping_add(view(0));
            g.ebx = g.ebx.wrapping_add(view(1));
            g.ecx = g.ecx.wrapping_add(view(2));
            g.eax = g.eax.wrapping_add(spread);
            g.ecx = g.ecx.wrapping_add(spread);
            for r in [&mut g.eax, &mut g.ebx, &mut g.ecx] {
                *r = (*r & 0x3_FFFF).wrapping_sub(0x1_FFFF);
            }
            let place = [
                x87_run::int_80(g.ecx as i32, false),
                x87_run::int_80(g.ebx as i32, false),
                x87_run::int_80(g.eax as i32, false),
            ];
            part(6, &mut g, &place)?;
            let s: u32 = game_read(SETTINGS_RVA);
            g.esi = s;
            if get(s + 0x10) != 0 {
                g.esi = k as u32;
                for (n, kept) in KEPT_RVA.into_iter().enumerate() {
                    let v: u32 = game_read(TURNED_RVA + 4 * n);
                    game_write(kept + 4 * k, x87::to_int(v));
                }
            }
        }
        game_write(DRAWN_RVA, 1u32);
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => {
                let left = crate::marks::place_view::place_view(r)?;
                *r = Regs { esp: r.esp, ..left };
            }
            2 => crate::scene::depth_buffer_on::depth_buffer(r, false),
            3 => {
                let (value, state) = (r.eax, r.ebx);
                crate::scene::set_render_state::set_render_state(r, value, state);
            }
            4 => crate::calls::call(CLEAR_TEXTURE_RVA, r),
            5 => {
                let left = crate::marks::drift_offsets::drift_offsets(r);
                *r = Regs { esp: r.esp, ..left };
            }
            _ => {
                use crate::marks::draw_streak as streak;
                let left = streak::draw_streak(r, [v[0], v[1], v[2]], &mut streak::game_part)?;
                *r = Regs { esp: r.esp, ..left };
            }
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { run_rain(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
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

extern "C" fn note_three(r: *const Regs, v: *const [u8; 10]) {
    note(6, unsafe { &*r }, unsafe {
        std::slice::from_raw_parts(v, 3)
    });
}

/// The test's stand-in for the streak: its three values taken off and noted.
#[unsafe(naked)]
unsafe extern "C" fn streak_stand_in() {
    core::arch::naked_asm!(
        "lea esp, [esp - 30]",
        "fstp tbyte ptr [esp]",
        "fstp tbyte ptr [esp + 10]",
        "fstp tbyte ptr [esp + 20]",
        "pushfd",
        "pushad",
        "mov eax, esp",
        "lea ecx, [esp + 36]",
        "push ecx",
        "push eax",
        "call {note}",
        "add esp, 8",
        "popad",
        "popfd",
        "lea esp, [esp + 30]",
        "ret",
        note = sym note_three,
    )
}

fn self_test() -> (usize, usize) {
    use crate::install::stand_ins::{set_handler, stand_in};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    set_handler(note_regs);
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x4C16F9)),
            0x42A,
            // mov esi, g_marksSettings
            &[0x8B, 0x35, 0x28, 0x7A, 0x76, 0x00],
            &[0xA8, 0xAD, 0xBC, 0xCB, 0xDA, 0xE9, 0xF8, 0xFD, 0x1C8, 0x3D7],
            &|c| match c + 0x40_0000 {
                0x4C1CC8 => Some(stand_in(1)),
                0x44E3F1 => Some(stand_in(2)),
                0x44EB72 => Some(stand_in(3)),
                0x44E958 => Some(stand_in(4)),
                0x4C1BA9 => Some(stand_in(5)),
                0x4C1DCD => Some(streak_stand_in as *const () as usize),
                _ => None,
            },
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut settings = vec![0u32; 0x2768 / 4];
    let sa = settings.as_mut_ptr() as u32;
    let regions: Vec<Region> = vec![
        (game_addr(rva(0x8BC405)) as u32, 0x8BDE55 - 0x8BC405),
        (game_addr(VERTICES_RVA) as u32, 0x80),
        at(COUNT_RVA),
        at(SCRATCH_RVA),
        at(STATUS_RVA),
    ];
    let mut vars = regions.clone();
    vars.extend([SETTINGS_RVA, RHW_RVA, DRIFTING_RVA, CLOCK_RVA].map(at));
    vars.push((game_addr(rva(0x8C5F91)) as u32, 0x28));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..5u32 {
        let k = case as f32;
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(SETTINGS_RVA, sa);
            let s = sa as *mut u32;
            for n in 0..0x40 {
                *s.add(n) = (n as u32).wrapping_mul(0x0101_0101) ^ case;
            }
            *s.add(2) = case % 5;
            *s.add(3) = u32::from(case % 2 == 0);
            *s.add(4) = u32::from(case != 1);
            *s.add(0x2764 / 4) = [0, 5, 0x30, 0x180, 0xFFFF_FFFF][case as usize];
            game_write(DRIFTING_RVA, u32::from(case >= 2));
            game_write(CLOCK_RVA, 0x1234_5678u32.wrapping_mul(case + 1));
            game_write(RHW_RVA, 3 + case);
            for n in 1..10 {
                game_write(
                    rva(0x8C5F91) + 4 * n,
                    (0.5f32 + n as f32 * 0.25 + k).to_bits(),
                );
            }
            for n in 0..3 {
                game_write(VIEW_RVA + 4 * n, 0x1_0000 * (n as u32 + case));
                game_write(TURNED_RVA + 4 * n, (n as f32 * 100.5 - k).to_bits());
            }
            std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0x80);
        };
        prepare();
        NOTED.lock().unwrap().clear();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            Some(regs(&ro)),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&regions),
        );
        prepare();
        let mine = unsafe {
            run_rain(&Regs::SENTINEL, &mut |p, r, v| {
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
    drop(settings);
    t.finish()
}
