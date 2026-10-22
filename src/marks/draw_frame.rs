//! `marks::draw_frame` (0x4C3EEE, through 0x4C4E28): the marks' drawing frame. Three
//! points (`flt_91FF4A` .., `flt_91FF56` .., `flt_91FF62` ..) turned by the view matrix
//! (`scene::rotate_point`) into `flt_8F39A9` .. (z, y, x of each on the x87 stack as the
//! original loads them); the course's corners (`g_courseInfo` + 0x1C2000, 0x8000 bytes)
//! into `g_marksCorners`; when `dword_621A80` changed since the last frame
//! (`dword_91FF6E`) `dword_8F3979` steps on (of 8); `dword_71C8E2` it, `dword_71C8E6` 0,
//! `dword_8F3981` 0; then `marks::cars_particles`, `marks::view_heading`,
//! `marks::view_motion`, `marks::begin_quads` (its time, `rdtsc`, in `flt_765124`), an
//! empty time in `flt_765118`, and `scene::depth_test_on`.
//!
//! In:  nothing
//! Out: the registers as `depth_test_on` leaves them
//!
//! Its calls through `part` (places 1 .. 5, those five), in the game their Rust versions.
//! Refuses (before anything) a null `g_courseInfo`, and `cars_particles`' refusals (what it
//! wrote before is put back first). Shadow mode runs the original alone (it tells the
//! device). Self-test: a copy whose five calls go to stand-ins (the three turns real),
//! against the Rust version on the same stand-ins; all but the two times compared.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::draw_frame", rva(0x4C3EEE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const VIEW_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC
/// (the point, where it goes turned).
const POINTS: [(usize, usize); 3] = [
    (rva(0x91FF4A), rva(0x8F39A9)),
    (rva(0x91FF56), rva(0x8F39B5)),
    (rva(0x91FF62), rva(0x8F39C1)),
];
const COURSE_INFO_RVA: usize = rva(0x5C6FA0); // g_courseInfo
const CORNERS_FROM: u32 = 0x1C_2000;
const CORNERS_RVA: usize = rva(0x8EB971); // g_marksCorners
const CORNERS_LEN: u32 = 0x8000;
const SOURCE_RVA: usize = rva(0x621A80); // dword_621A80
const LAST_SOURCE_RVA: usize = rva(0x91FF6E); // dword_91FF6E
const STEP_RVA: usize = rva(0x8F3979); // dword_8F3979: of 8
const SOUND_RVA: [usize; 2] = [rva(0x71C8E2), rva(0x71C8E6)];
const CLEARED_RVA: usize = rva(0x8F3981); // dword_8F3981
const FRAME_CYCLES_RVA: usize = rva(0x765114); // dword_765114
const TIMES: [(usize, usize); 2] = [
    (rva(0x91FF9A), rva(0x765124)),
    (rva(0x91FF9E), rva(0x765118)),
];

/// A call by place: 1 `cars_particles`, 2 `view_heading`, 3 `view_motion`, 4
/// `begin_quads`, 5 `depth_test_on`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn rdtsc() -> u32 {
    unsafe { core::arch::x86::_rdtsc() as u32 }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let info: u32 = game_read(COURSE_INFO_RVA);
        if info == 0 {
            return Err(BadInput("no course info (g_courseInfo 0)").into());
        }
        let m = game_addr(VIEW_RVA) as u32;
        for (from, to) in POINTS {
            let p: [u32; 3] = game_read(from);
            let turned = rotate_point(m, [extended(p[2]), extended(p[1]), extended(p[0])]);
            for (k, v) in turned.iter().enumerate() {
                game_write(to + 4 * k, rounded(*v));
            }
            g.esi = m;
        }
        g.esi = info + CORNERS_FROM;
        g.edi = game_addr(CORNERS_RVA) as u32;
        std::ptr::copy(g.esi as *const u8, g.edi as *mut u8, CORNERS_LEN as usize);
        g.esi += CORNERS_LEN;
        g.edi += CORNERS_LEN;
        g.ecx = 0;
        g.eax = game_read(SOURCE_RVA);
        if g.eax != game_read::<u32>(LAST_SOURCE_RVA) {
            game_write(STEP_RVA, game_read::<u32>(STEP_RVA).wrapping_add(1) & 7);
        }
        game_write(LAST_SOURCE_RVA, g.eax);
        game_write(SOUND_RVA[0], g.eax);
        game_write(SOUND_RVA[1], 0u32);
        game_write(CLEARED_RVA, 0u32);
        for place in 1..=3 {
            part(place, &mut g)?;
        }
        let start = rdtsc();
        part(4, &mut g)?;
        let frame: i32 = game_read(FRAME_CYCLES_RVA);
        let took = rdtsc().wrapping_sub(start) as i32;
        game_write(
            TIMES[0].1,
            x87::int_over_int_times(took, frame, game_read(TIMES[0].0)),
        );
        let start = rdtsc();
        let took = rdtsc().wrapping_sub(start) as i32;
        game_write(
            TIMES[1].1,
            x87::int_over_int_times(took, frame, game_read(TIMES[1].0)),
        );
        part(5, &mut g)?;
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::{begin_quads, cars_particles, view_heading, view_motion};
    unsafe {
        let left = match place {
            1 => cars_particles::cars_particles(r)?,
            2 => view_heading::view_heading(r),
            3 => view_motion::view_motion(r),
            4 => begin_quads::begin_quads(r, &mut begin_quads::game_part)?,
            _ => {
                crate::scene::depth_test_on::depth_test_on(r);
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
    match unsafe { draw_frame(r, &mut game_part) } {
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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { draw_frame(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}

fn regions() -> Vec<Region> {
    let mut r: Vec<Region> = POINTS
        .iter()
        .map(|&(_, to)| (game_addr(to) as u32, 12))
        .collect();
    r.push((game_addr(CORNERS_RVA) as u32, CORNERS_LEN as usize));
    r.extend(
        [
            LAST_SOURCE_RVA,
            STEP_RVA,
            SOUND_RVA[0],
            SOUND_RVA[1],
            CLEARED_RVA,
        ]
        .map(at),
    );
    r
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(COURSE_INFO_RVA) } == 0 {
        return t.skip("no course info");
    }
    // fld flt_91FF4A
    let entry: &[u8] = &[0xD9, 0x05, 0x4A, 0xFF, 0x91, 0x00];
    let copy = match copy(
        0x4C3EEE,
        0x196,
        entry,
        &[0x18, 0x47, 0x76, 0x118, 0x11D, 0x122, 0x133, 0x190],
        &[
            (0x4C9AF6, 1),
            (0x4C4CEC, 2),
            (0x4C4D40, 3),
            (0x4C4453, 4),
            (0x44E3E1, 5),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend([SOURCE_RVA, TIMES[0].1, TIMES[1].1].map(at));
    vars.extend(POINTS.iter().map(|&(from, _)| (game_addr(from) as u32, 12)));
    let saved = capture(&vars);
    for case in 0..3u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(
                    SOURCE_RVA,
                    if case == 1 {
                        0x55u32
                    } else {
                        game_read::<u32>(LAST_SOURCE_RVA)
                    },
                );
                for (n, &(from, _)) in POINTS.iter().enumerate() {
                    for k in 0..3 {
                        game_write(
                            from + 4 * k,
                            ((n * 3 + k) as f32 * 1.5 - 4.0 + case as f32).to_bits(),
                        );
                    }
                }
            },
            &|r| unsafe { draw_frame(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
