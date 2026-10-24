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
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `marks::run_frame` (0x4D1569). Its calls through `part` (1 `place_view`, 2
//! `depth_buffer_off`, 3 a render state, 4 `clear_texture`, 5 `drift_offsets`, 6
//! `draw_streak` with three values); in the game their Rust versions (`clear_texture`, the
//! platform's, through the game: section 15). The x87 blocks are the original's
//! instructions over the game's variables. Refuses null settings and a view entry past
//! the five (the settings' +8), before anything. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose calls go to stand-ins (the streak's taking its values
//! off), against the Rust version, on the game's tables with settings of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::marks::setup::SETTINGS_RVA;
use crate::math::{x87, x87_run};
use crate::scene::vector3::STATUS_RVA;

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
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];
const COLOURS: [u32; 4] = [0xFF_FFFF, 0, 0x2800_0000, 0x28FF_FFFF];

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

/// The Rust version (see the header).
pub unsafe fn run_rain() -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        if get(s + 8) >= ENTRIES {
            return Err(BadInput("a rain view entry past the five").into());
        }
        let b = game_addr(0) as u32;
        let mut drops = get(s + 0x2764) as i32;
        game_write(DROPS_RVA, drops);
        if drops == 0 {
            game_write(DRAWN_RVA, 0u32);
            return Ok(());
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
        crate::marks::place_view::place_view()?;
        crate::scene::depth_buffer_on::depth_buffer(false);
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        // `clear_texture`: what it returns went only to eax, which nothing here reads.
        crate::scene::clear_texture::clear_texture();
        let rhw: u32 = game_read(RHW_RVA);
        for (n, colour) in COLOURS.into_iter().enumerate() {
            let v = VERTICES_RVA + 0x20 * n;
            game_write(SCRATCH_RVA, rhw);
            game_write(v + 0xC, x87::int_to_float(rhw as i32));
            game_write(v + 0x10, colour);
            for o in [0x14, 0x18, 0x1C] {
                game_write(v + o, 0u32);
            }
        }
        game_write(COUNT_RVA, 4u32);
        if game_read::<u32>(DRIFTING_RVA) != 0 {
            crate::marks::drift_offsets::drift_offsets();
        }
        let s: u32 = game_read(SETTINGS_RVA);
        let e = get(s + 8) * 12;
        if get(s + 0xC) != 0 || game_read::<u32>(DRIFTING_RVA) != 0 {
            box_moved(b, e);
        }
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
            let clock = game_read::<u32>(CLOCK_RVA) & 0xF_FFFF;
            game_write(
                DRIFT_RVA,
                x87::int_times_to_int(clock as i32, game_read(DRIFT_K_RVA)),
            );
        }
        for k in (0..drops.max(0) as usize).rev() {
            let drift: u32 = game_read(DRIFT_RVA);
            let spread: u32 = game_read(SPREAD_RVA);
            let view = |n: usize| game_read::<u32>(VIEW_RVA + 4 * n);
            let noise = |n: usize| game_read::<u32>(NOISE_RVA[n] + 4 * k);
            let x = noise(0).wrapping_add(view(0)).wrapping_add(spread);
            let y = noise(1).wrapping_add(drift).wrapping_add(view(1));
            let z = noise(2).wrapping_add(view(2)).wrapping_add(spread);
            let [x, y, z] = [x, y, z].map(|v| (v & 0x3_FFFF).wrapping_sub(0x1_FFFF));
            let place = [
                x87_run::int_80(z as i32, false),
                x87_run::int_80(y as i32, false),
                x87_run::int_80(x as i32, false),
            ];
            crate::marks::draw_streak::draw_streak(place)?;
            let s: u32 = game_read(SETTINGS_RVA);
            if get(s + 0x10) != 0 {
                for (n, kept) in KEPT_RVA.into_iter().enumerate() {
                    let v: u32 = game_read(TURNED_RVA + 4 * n);
                    game_write(kept + 4 * k, x87::to_int(v));
                }
            }
        }
        game_write(DRAWN_RVA, 1u32);
    }
    Ok(())
}
