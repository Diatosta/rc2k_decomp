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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

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
            4 => begin_quads::begin_quads(r)?,
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
