//! `marks::draw_mark_slots` (0x4C969A): the mark slots drawn, one square sprite each. When
//! the settings' +0x8C is set, `marks::add_mark_slot` first (eax the settings' +0x64, esi
//! `flt_5EA3CC`); `scene::set_texture` with `dword_5D1E10`. Then for each slot of
//! `g_markSlots` from 0x20 down to 0 (one past the 0x20 that `add_mark_slot` fills:
//! section 16) whose life (+0) is above 0:
//! - the life less `g_marksFrameTicks`;
//! - its point (+4, 12 bytes each at `flt_769312`) as z, y, -x into `dword_9352F4` ..
//!   (z times `flt_9353EE`, (y - `flt_9353F2`) times `flt_9353F6`, -x times `flt_9353FA`),
//!   turned by the slot's matrix (+8) and the view (`scene::rotate_point` twice);
//! - one of eight corners of `g_marksCorners` by the slots' random (`dword_9353FE` times
//!   0xC5 plus 0x5D; entry 0x50 plus its low three bits): the four vertices' texture places
//!   (its words, times `flt_8EB969`), colours -1;
//! - unless the turned z is at or below `flt_5C5E18` (a signed compare of the bits): the
//!   point projected (`flt_624088` over z; centre `flt_62409C`, `flt_6240A0`), half the
//!   square q times `flt_934F8C` (slot 0's +0x10 for every slot: section 16) in
//!   `flt_9353A6`, the four vertices around it, their z (times `flt_8EB96D`) and rhw (1 /
//!   z times `flt_935402`), and `scene::draw_fan` (the vertex count as it is).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as the last slot's path leaves them (ebp 0 when slot 0 is not live)
//!
//! Caller: `marks::run_frame` (0x4D1659). Its calls through `part` (1 `add_mark_slot`, 2
//! `set_texture`, 3 `draw_fan`); in the game their Rust versions (the device's through
//! the game). Refuses (before anything) null settings and a live slot with a null matrix.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose three calls
//! go to stand-ins (the turns real; each draw notes the four vertices), against the Rust
//! version on the same, on settings, slots, points and matrices of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::setup::{SETTINGS_RVA, SLOT_LEN, SLOTS_RVA};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::draw_mark_slots", rva(0x4C969A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The slots it goes through: 0x1F down to 0 (`push 20h`, then `dec` before the first).
const DRAWN: u32 = 0x20;
const PLACE_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC: add_mark_slot's esi
const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const TICKS_RVA: usize = rva(0x764FD0); // g_marksFrameTicks
const POINTS_RVA: usize = rva(0x769312); // flt_769312: 12 bytes each
const WORK_RVA: usize = rva(0x9352F4); // dword_9352F4 .. dword_9352FC
/// flt_9353EE, flt_9353F2, flt_9353F6, flt_9353FA.
const PLACE_K: [usize; 4] = [rva(0x9353EE), rva(0x9353F2), rva(0x9353F6), rva(0x9353FA)];
const VIEW_RVA: usize = rva(0x5EA2CC); // g_viewMatrix
const RANDOM_RVA: usize = rva(0x9353FE); // dword_9353FE
const CORNERS_RVA: usize = rva(0x8EB971); // g_marksCorners
const UV_K_RVA: usize = rva(0x8EB969); // flt_8EB969
const NEAR_RVA: usize = rva(0x5C5E18); // flt_5C5E18
const FOCAL_RVA: usize = rva(0x624088); // flt_624088
const CENTRE_RVA: [usize; 2] = [rva(0x62409C), rva(0x6240A0)];
const SIZE_K_RVA: usize = rva(0x934F8C); // flt_934F8C: slot 0's +0x10
const HALF_RVA: usize = rva(0x9353A6); // flt_9353A6
const Z_K_RVA: usize = rva(0x8EB96D); // flt_8EB96D
const RHW_K_RVA: usize = rva(0x935402); // flt_935402
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices: TLVertex[4]
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

/// A call by place: 1 `add_mark_slot` (eax, esi), 2 `set_texture` (eax), 3 `draw_fan`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

fn slot(k: u32) -> u32 {
    game_addr(SLOTS_RVA) as u32 + k * SLOT_LEN
}

/// Why it refuses, if it does, worked out before anything is written.
unsafe fn refusal() -> Option<&'static str> {
    unsafe {
        if game_read::<u32>(SETTINGS_RVA) == 0 {
            return Some("no marks settings");
        }
        for k in 0..DRAWN {
            if (get(slot(k)) as i32) > 0 && get(slot(k) + 8) == 0 {
                return Some("a live mark slot with a null matrix");
            }
        }
        None
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_mark_slots(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    if let Some(why) = unsafe { refusal() } {
        return Err(BadInput(why).into());
    }
    let mut g = *entry;
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        g.esi = settings;
        if get(settings + 0x8C) != 0 {
            g.esi = game_addr(PLACE_RVA) as u32;
            g.eax = get(settings + 0x64);
            part(1, &mut g)?;
        }
        g.eax = game_read(TEXTURE_RVA);
        part(2, &mut g)?;
        for k in (0..DRAWN).rev() {
            g.ebp = k * SLOT_LEN;
            let s = slot(k);
            if (get(s) as i32) <= 0 {
                continue;
            }
            g.eax = game_read(TICKS_RVA);
            (s as *mut u32).write_unaligned(get(s).wrapping_sub(g.eax));
            let p = (game_addr(POINTS_RVA) as u32).wrapping_add(get(s + 4).wrapping_mul(12));
            [g.eax, g.ebx, g.ecx] = [get(p), get(p + 4), get(p + 8)];
            g.eax ^= 0x8000_0000;
            let k4 = PLACE_K.map(|r| game_read::<u32>(r));
            let z = x87::mul(g.ecx, k4[0]);
            let y = crate::math::x87_run::sub_mul(g.ebx, k4[1], k4[2]);
            let x = x87::mul(g.eax, k4[3]);
            g.esi = get(s + 8);
            let once = rotate_point(g.esi, [extended(z), extended(y), extended(x)]);
            g.esi = game_addr(VIEW_RVA) as u32;
            let twice = rotate_point(g.esi, once);
            let [z, y, x] = twice.map(rounded);
            for (n, v) in [z, y, x].into_iter().enumerate() {
                game_write(WORK_RVA + 4 * n, v);
            }
            let r = game_read::<u32>(RANDOM_RVA)
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            game_write(RANDOM_RVA, r);
            g.eax = game_addr(CORNERS_RVA) as u32 + ((r & 7) + 0x50) * 0x80;
            let word = |off: u32| ((g.eax + off) as *const u16).read_unaligned() as u32;
            let uv_k: u32 = game_read(UV_K_RVA);
            for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
                let vertex = VERTICES_RVA + 0x20 * n;
                game_write(vertex + 0x18, x87::int_times(word(u) as i32, uv_k));
                game_write(vertex + 0x1C, x87::int_times(word(v) as i32, uv_k));
            }
            [g.ebp, g.edi] = [word(8), word(0xA)];
            for n in 0..4 {
                game_write(VERTICES_RVA + 0x20 * n + 0x10, 0xFFFF_FFFFu32);
            }
            g.ecx = x;
            if (x as i32) <= game_read::<i32>(NEAR_RVA) {
                continue;
            }
            let [px, py, half] = x87::slot_project(
                game_read(FOCAL_RVA),
                x,
                z,
                game_read(CENTRE_RVA[0]),
                y,
                game_read(CENTRE_RVA[1]),
                game_read(SIZE_K_RVA),
            );
            game_write(WORK_RVA, px);
            game_write(WORK_RVA + 4, py);
            game_write(HALF_RVA, half);
            let corners = [
                (x87::sub(px, half), x87::sub(py, half)),
                (x87::add(px, half), x87::sub(py, half)),
                (x87::add(px, half), x87::add(py, half)),
                (x87::sub(px, half), x87::add(py, half)),
            ];
            let depth = x87::mul(x, game_read(Z_K_RVA));
            let rhw = x87::reciprocal_times(x, game_read(RHW_K_RVA));
            for (n, (vx, vy)) in corners.into_iter().enumerate() {
                let vertex = VERTICES_RVA + 0x20 * n;
                game_write(vertex, vx);
                game_write(vertex + 4, vy);
                game_write(vertex + 8, depth);
                game_write(vertex + 0xC, rhw);
            }
            part(3, &mut g)?;
        }
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::marks::add_mark_slot::add_mark_slot(r.eax, r.esi),
            2 => crate::scene::set_texture::set_texture_in_game(r),
            _ => crate::scene::draw_fan::draw_fan(r, &mut crate::scene::draw_fan::game_part),
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
    match unsafe { draw_mark_slots(r, &mut game_part) } {
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

/// What it writes.
fn regions() -> Vec<Region> {
    let mut r = vec![
        (slot(0), ((DRAWN + 1) * SLOT_LEN) as usize),
        (game_addr(WORK_RVA) as u32, 12),
        (game_addr(VERTICES_RVA) as u32, 0x80),
    ];
    r.extend([RANDOM_RVA, HALF_RVA].map(at));
    r
}

/// The vertices at each draw, both sides.
static DRAWS: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());

/// The test's stand-ins: noted, and each draw's vertices kept.
fn note(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    if place == 3 {
        let v = capture(&[(game_addr(VERTICES_RVA) as u32, 0x80)]);
        DRAWS.lock().unwrap().push(v);
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov esi, g_marksSettings
    let entry: &[u8] = &[0x8B, 0x35, 0x28, 0x7A, 0x76, 0x00];
    let copy = match copy(
        0x4C969A,
        0x35A,
        entry,
        &[0x1D, 0x27, 0xD6, 0xE1, 0x347],
        &[(0x4C99F4, 1), (0x44E77E, 2), (0x44EA65, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note);
    let mut settings = vec![0u32; 0x40];
    let sa = settings.as_mut_ptr() as u32;
    let matrices: Vec<u32> = (0..2 * 16)
        .map(|n| ((n % 7) as f32 * 0.35 - 0.8 + (n / 16) as f32 * 0.1).to_bits())
        .collect();
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend(
        [
            SETTINGS_RVA,
            TICKS_RVA,
            NEAR_RVA,
            FOCAL_RVA,
            CENTRE_RVA[0],
            CENTRE_RVA[1],
            UV_K_RVA,
            Z_K_RVA,
            RHW_K_RVA,
        ]
        .map(at),
    );
    vars.extend(PLACE_K.map(at));
    vars.push((game_addr(VIEW_RVA) as u32, 0x40));
    vars.push((game_addr(POINTS_RVA) as u32, 8 * 12));
    vars.push((game_addr(CORNERS_RVA) as u32 + 0x50 * 0x80, 8 * 0x80));
    let saved = capture(&vars);
    let draws = |what: &str, t: &mut SelfTest, theirs: Vec<Vec<u8>>, mine: Vec<Vec<u8>>| {
        t.check(theirs == mine, || {
            format!(
                "{what}: the draws' vertices differ ({} draws, rust {}); first {:x?} / {:x?}",
                theirs.len(),
                mine.len(),
                theirs.first(),
                mine.first()
            )
        });
    };
    for case in 0..4u32 {
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(SETTINGS_RVA, sa);
            let s = sa as *mut u32;
            *s.add(0x8C / 4) = u32::from(case % 2 == 1);
            *s.add(0x64 / 4) = 3 + case;
            game_write(TICKS_RVA, 2 + case);
            game_write(NEAR_RVA, 1.0f32.to_bits());
            game_write(FOCAL_RVA, 300.0f32.to_bits());
            game_write(CENTRE_RVA[0], 320.0f32.to_bits());
            game_write(CENTRE_RVA[1], 240.0f32.to_bits());
            game_write(UV_K_RVA, (1.0f32 / 256.0).to_bits());
            game_write(Z_K_RVA, 0.001f32.to_bits());
            game_write(RHW_K_RVA, 1.5f32.to_bits());
            for (n, k) in PLACE_K.into_iter().enumerate() {
                game_write(k, (0.5 + n as f32 * 0.25).to_bits());
            }
            for n in 0..16 {
                game_write(VIEW_RVA + 4 * n, matrices[16 + n]);
            }
            // The view's place: points well in front.
            game_write(VIEW_RVA + 0x38, (20.0f32 + case as f32 * 30.0).to_bits());
            for n in 0..8 * 3 {
                let v = (n as f32 * 1.7 - 9.0) * (1.0 + case as f32);
                game_write(POINTS_RVA + 4 * n, v.to_bits());
            }
            let corners = game_addr(CORNERS_RVA) as u32 + 0x50 * 0x80;
            for n in 0..8 * 0x40 {
                ((corners + 2 * n) as *mut u16).write_unaligned((n * 37 + case) as u16);
            }
            game_write(RANDOM_RVA, case.wrapping_mul(0x9E37_79B9));
            for k in 0..=DRAWN {
                let p = slot(k) as *mut u32;
                // Lives: some dead, some live; the last slot (one past) live in case 3.
                let life = match (k + case) % 4 {
                    0 => 0,
                    1 => 0xFFFF_FFF0,
                    _ => 5 + k,
                };
                *p = if k == DRAWN && case != 3 { 0 } else { life };
                *p.add(1) = k % 8;
                *p.add(2) = matrices.as_ptr() as u32;
                *p.add(3) = 0x77;
                *p.add(4) = (0.5f32 + k as f32 * 0.1).to_bits();
            }
            std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0x80);
        };
        DRAWS.lock().unwrap().clear();
        let mut theirs = Vec::new();
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| {
                prepare();
                // The original's draws are in when the Rust side's set-up starts.
                let mut d = DRAWS.lock().unwrap();
                if !d.is_empty() {
                    THEIRS.lock().unwrap().append(&mut d);
                }
            },
            &|r| unsafe {
                draw_mark_slots(r, &mut |p, g| {
                    note(p, g);
                    Ok(())
                })
            },
        );
        theirs.append(&mut THEIRS.lock().unwrap());
        let mine = std::mem::take(&mut *DRAWS.lock().unwrap());
        draws(&format!("case {case}"), &mut t, theirs, mine);
    }
    restore(&vars, &saved);
    drop(settings);
    t.finish()
}

/// The original's draws, moved aside when the Rust side's set-up starts.
static THEIRS: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
