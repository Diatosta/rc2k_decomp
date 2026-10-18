//! `hud::draw_view_strip` (0x45B86E): three flat quads side by side for view ebp (a reading:
//! a strip of the screen's overlay): top `dword_6D1880`[ebp] (or [ebp + 6] with
//! `dword_6D1502` 0) and height 0x19 scaled from 600 to `g_screenHeight`, left
//! `dword_6D18B0`[ebp] and width 0x78 scaled from 800 to `g_screenWidth` (into
//! `dword_6D1870` .. `dword_6D187C`), texture rows 0x320000 apart; then the shared tail
//! (`overlay_tail`, 0x45B9AA, which `hud::draw_view_strips` jumps to as well): with
//! `dword_6D14F6` bit 0x40 and `dword_6238A4` 0, two more quads at the top corners
//! (texture `dword_6240BC` 0xC4).
//!
//! In:  ebp the view (0 .. 5)
//! Out: eax / ebx / ebp / ecx / edx as the last quad's set-up leaves them (see the code)
//!
//! Callers: `hud::draw_view_strips` (twice). Each quad's inputs (`dword_623DE8` ..
//! `dword_623E24`, `g_polygonAlpha` 0xFF, colour -1, `dword_6240E0` 1, `dword_6240E4` 0)
//! then `scene::draw_flat_quad` through `part` (1), in the game its Rust version (inside
//! `pusha` .. `popa`). Refuses a view past 5 (the tables' end), before anything. Shadow
//! mode runs the original alone (it draws). Self-test: a copy whose quads go to a
//! stand-in noting their inputs, against the Rust version, views and settings by lot.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_view_strip", rva(0x45B86E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TOPS_RVA: usize = rva(0x6D1880); // dword_6D1880: 12
const LEFTS_RVA: usize = rva(0x6D18B0); // dword_6D18B0: 6
const SIZES_RVA: usize = rva(0x6D1870); // dword_6D1870 .. dword_6D187C: top, height, left, width
const WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
const FLAT_RVA: usize = rva(0x6D1502); // dword_6D1502
pub const QUAD_IN_RVA: usize = rva(0x623DE8); // dword_623DE8 .. dword_623E24
const ALPHA_RVA: usize = rva(0x6240D0); // g_polygonAlpha
pub const TEXTURE_RVA: usize = rva(0x6240BC); // dword_6240BC
const SIDES_RVA: [usize; 2] = [rva(0x6240E0), rva(0x6240E4)];
const SHOWN_RVA: usize = rva(0x6D14F6); // dword_6D14F6
const PLAYERS_RVA: usize = rva(0x6238A4); // dword_6238A4
const VIEWS: u32 = 6;

/// A call by place: 1 `draw_flat_quad`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn set(r: usize, v: u32) {
    unsafe { game_write(r, v) };
}

/// `v` times the screen's `size` over `of` (`mul`, `div`): eax the quotient, edx the rest.
fn scaled(g: &mut Regs, v: u32, size: usize, of: u32) -> u32 {
    let n = u64::from(v) * u64::from(unsafe { game_read::<u32>(size) });
    g.eax = (n / u64::from(of)) as u32;
    g.edx = (n % u64::from(of)) as u32;
    g.ebx = of;
    g.eax
}

/// The four sizes (top, height, left, width) into `dword_6D1870` ..: top and height from
/// 600 to the screen's height, left and width from 800 to its width.
fn sizes(g: &mut Regs, [top, height, left, width]: [u32; 4]) {
    let v = [
        scaled(g, top, HEIGHT_RVA, 0x258),
        scaled(g, height, HEIGHT_RVA, 0x258),
        scaled(g, left, WIDTH_RVA, 0x320),
        scaled(g, width, WIDTH_RVA, 0x320),
    ];
    for (n, x) in v.into_iter().enumerate() {
        set(SIZES_RVA + 4 * n, x);
    }
}

/// One quad at `ebx` (left), the sizes' top and height, `ebp` wide, texture rows from
/// `ecx` (inside `pusha` .. `popa`); `texture` into `dword_6240BC` when some.
fn quad(g: &Regs, texture: Option<u32>, part: &mut Part) -> Result<(), Refused> {
    let mut r = *g;
    let get = |n: usize| unsafe { game_read::<u32>(SIZES_RVA + 4 * n) };
    let q = |o: usize, v: u32| set(QUAD_IN_RVA + o, v);
    q(0x28, r.ebx);
    q(0x2C, r.ebx.wrapping_add(r.ebp));
    q(0x38, get(0));
    q(0x3C, get(0).wrapping_add(get(1)));
    for (n, v) in [0x8_0000u32, 0xF8_0000, 0xF8_0000, 0x8_0000]
        .into_iter()
        .enumerate()
    {
        q(8 + 4 * n, v);
    }
    q(0x18, r.ecx);
    q(0x1C, r.ecx);
    r.eax = r.ecx.wrapping_add(0x32_0000);
    q(0x20, r.eax);
    q(0x24, r.eax);
    if let Some(t) = texture {
        set(TEXTURE_RVA, t);
    }
    set(ALPHA_RVA, 0xFF);
    q(0, 0xFFFF_FFFF);
    set(SIDES_RVA[0], 1);
    set(SIDES_RVA[1], 0);
    part(1, &mut r)
}

/// The shared tail (0x45B9AA, see the header).
pub unsafe fn overlay_tail(g: &mut Regs, part: &mut Part) -> Result<(), Refused> {
    if unsafe { game_read::<u32>(SHOWN_RVA) } & 0x40 == 0
        || unsafe { game_read::<u32>(PLAYERS_RVA) } != 0
    {
        return Ok(());
    }
    for (left, rows) in [(4u32, 0x96_0000u32), (0x2B4, 0xC8_0000)] {
        sizes(g, [2, 0xF, left, 0x44]);
        let get = |n: usize| unsafe { game_read::<u32>(SIZES_RVA + 4 * n) };
        [g.ebx, g.ebp, g.edx, g.ecx] = [get(2), get(3), 0, rows];
        quad(g, Some(0xC4), part)?;
    }
    Ok(())
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_view_strip(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    let view = g.ebp;
    if view >= VIEWS {
        return Err(BadInput("a view past the overlay's tables").into());
    }
    unsafe {
        let shift = if game_read::<u32>(FLAT_RVA) == 0 {
            6
        } else {
            0
        };
        let top: u32 = game_read(TOPS_RVA + 4 * (view + shift) as usize);
        let left: u32 = game_read(LEFTS_RVA + 4 * view as usize);
        sizes(&mut g, [top, 0x19, left, 0x78]);
        let get = |n: usize| game_read::<u32>(SIZES_RVA + 4 * n);
        [g.ebx, g.ebp, g.ecx, g.edx] = [get(2), get(3), 0, 0];
        for _ in 0..3 {
            quad(&g, None, &mut part)?;
            g.ebx = g.ebx.wrapping_add(get(3));
            g.ecx = g.ecx.wrapping_add(0x32_0000);
            g.edx += 1;
        }
        overlay_tail(&mut g, &mut part)?;
    }
    Ok(g)
}

/// In the game: `draw_flat_quad`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::scene::draw_flat_quad as fq;
    *r = unsafe { fq::draw_flat_quad(r, &mut fq::game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_view_strip(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// A quad as the tests note it: the place, eax .. ebp, its inputs and texture.
type Noted = (u32, [u32; 6], [u32; 0x10], u32);

static NOTED: Mutex<Vec<Noted>> = Mutex::new(Vec::new());

/// For `install::stand_ins::set_handler` and the Rust side of the tests.
pub fn note_quad(place: u32, r: &mut Regs) {
    let inputs: [u32; 0x10] = unsafe { game_read(QUAD_IN_RVA) };
    let texture: u32 = unsafe { game_read(TEXTURE_RVA) };
    NOTED.lock().unwrap().push((
        place,
        [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.ebp],
        inputs,
        texture,
    ));
}

pub fn take() -> Vec<Noted> {
    std::mem::take(&mut *NOTED.lock().unwrap())
}

/// What the strips write (for the tests).
pub fn regions() -> Vec<Region> {
    vec![
        (game_addr(SIZES_RVA) as u32, 0x10),
        (game_addr(QUAD_IN_RVA) as u32, 0x40),
        at(ALPHA_RVA),
        at(TEXTURE_RVA),
        at(SIDES_RVA[0]),
        at(SIDES_RVA[1]),
    ]
}

/// The settings by lot (for the tests): what they held is the caller's to put back
/// (`settings()`).
pub unsafe fn test_settings(case: u32) {
    set(FLAT_RVA, case & 1);
    set(SHOWN_RVA, if case.is_multiple_of(3) { 0 } else { 0x40 });
    set(PLAYERS_RVA, u32::from(case == 4));
    set(TEXTURE_RVA, 0xC5);
}

pub fn settings() -> Vec<Region> {
    [FLAT_RVA, SHOWN_RVA, PLAYERS_RVA].map(at).to_vec()
}

fn self_test() -> (usize, usize) {
    use crate::install::stand_ins::set_handler;
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, 0
    let entry: &[u8] = &[0xB8, 0x00, 0x00, 0x00, 0x00];
    let copy = match copy(
        0x45B86E,
        0x38C,
        entry,
        &[0x120, 0x26C, 0x385],
        &[(0x44AF85, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    set_handler(note_quad);
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend(settings());
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..6u32 {
        let before = Regs {
            ebp: case % VIEWS,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            test_settings(case);
        };
        prepare();
        take();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take(), capture(&regions));
        prepare();
        let mine = unsafe {
            draw_view_strip(&before, &mut |p, r| {
                note_quad(p, r);
                Ok(())
            })
        };
        let mine = (mine.as_ref().ok().map(regs), take(), capture(&regions));
        t.check(theirs == mine, || {
            format!("case {case}: original {:x?} {} quads; rust {:x?} {} quads; same quads {}; same state {}", theirs.0, theirs.1.len(), mine.0, mine.1.len(), theirs.1 == mine.1, theirs.2 == mine.2)
        });
    }
    restore(&vars, &saved);
    t.finish()
}
