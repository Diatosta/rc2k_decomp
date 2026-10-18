//! The tyre-change screen's drawing (stream B, 2026-10-18): `drive::tyre_change_screen`
//! (0x445047), `drive::tyre_screen` (0x41C5FA), `drive::tyre_screen_parts` (0x41C790),
//! `drive::tyre_turned_quad` (0x41C969), `drive::tyre_quad` (0x41DD3B). Their calls by
//! place: 0 `d3d::set_texture` (0x41ACF5), 1 `d3d::set_render_state` (0x41AB5B), 2
//! `tyre_quad`, 3 `tyre_turned_quad`, 4 `tyre_screen_parts`, 5 the device's
//! DrawPrimitive (`sub_41ADE1`: platform, a call into the game), 6 `tyre_screen`. In the
//! game the Rust versions and the game's three; in the self-tests copies whose calls go
//! to stand-ins noting them (with the vertices for 5), against the Rust versions with the
//! same.
//!
//! This file: the places, the quads' shared steps and the tests' frame.

use crate::errors::Refused;
use crate::framework::{Regs, SelfTest, call_game, game_addr, rva};
use crate::physics::flow::{a, base, p, x87_run};
use crate::race::car_starts::Part;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture};
use std::sync::Mutex;

pub const TEXTURE: u32 = 0;
pub const STATE: u32 = 1;
pub const QUAD: u32 = 2;
pub const TURNED: u32 = 3;
pub const PARTS: u32 = 4;
pub const DRAW: u32 = 5;
pub const SCREEN: u32 = 6;

/// The places' functions (IDA addresses), by place.
pub const PLACES: [usize; 7] = [
    0x41ACF5, 0x41AB5B, 0x41DD3B, 0x41C969, 0x41C790, 0x41ADE1, 0x41C5FA,
];

/// `g_polygonVertices`: the four vertices (0x20 bytes each) a quad draws.
pub const VERTS: usize = 0x9605A0;

/// The calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            QUAD => *r = crate::drive::tyre_quad::tyre_quad(r, &mut game_part)?,
            TURNED => *r = crate::drive::tyre_turned_quad::tyre_turned_quad(r, &mut game_part)?,
            PARTS => *r = crate::drive::tyre_screen_parts::tyre_screen_parts(r, &mut game_part)?,
            SCREEN => *r = crate::drive::tyre_screen::tyre_screen(r, &mut game_part)?,
            _ => call_game(r, game_addr(rva(PLACES[place as usize]))),
        }
    }
    Ok(())
}

/// The quads' fixed words (0x41DD68 .. / 0x41CA49 ..): z 0, rhw 1.0, the colour, the
/// specular 0.
pub fn quad_words(colour: u32) {
    for k in 0..4 {
        let v = VERTS + 0x20 * k;
        p(v + 8, 0);
        p(v + 0xC, 0x3F80_0000);
        p(v + 0x10, colour);
        p(v + 0x14, 0);
    }
}

/// The quads' shared tail (0x41DDA1 ..): the texture places from the rectangle at `uv`
/// (x, y, w, h, whole numbers) x `flt_5C2C64`, and the places scaled to the screen
/// (`g_screenWidth` x `flt_5C2C68`, `g_screenHeight` x `flt_5C2C6C`, through
/// `dword_5183CC`).
pub fn quad_tail(uv: u32) {
    x87_run!(
        "fild dword ptr [{s}]",
        "fmul dword ptr [{b} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x18]",
        "fstp dword ptr [{v} + 0x58]",
        "fild dword ptr [{s}]",
        "fiadd dword ptr [{s} + 0x8]",
        "fmul dword ptr [{b} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x38]",
        "fstp dword ptr [{v} + 0x78]",
        "fild dword ptr [{s} + 0x4]",
        "fmul dword ptr [{b} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x1C]",
        "fstp dword ptr [{v} + 0x3C]",
        "fild dword ptr [{s} + 0x4]",
        "fiadd dword ptr [{s} + 0xC]",
        "fmul dword ptr [{b} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x5C]",
        "fstp dword ptr [{v} + 0x7C]",
        "fild dword ptr [{b} + 0xF494C]",
        "fmul dword ptr [{b} + 0x1C2C68]",
        "fstp dword ptr [{b} + 0x1183CC]",
        "fld dword ptr [{v}]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v}]",
        "fld dword ptr [{v} + 0x20]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x20]",
        "fld dword ptr [{v} + 0x40]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x40]",
        "fld dword ptr [{v} + 0x60]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x60]",
        "fild dword ptr [{b} + 0xF4950]",
        "fmul dword ptr [{b} + 0x1C2C6C]",
        "fstp dword ptr [{b} + 0x1183CC]",
        "fld dword ptr [{v} + 0x4]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x4]",
        "fld dword ptr [{v} + 0x24]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x24]",
        "fld dword ptr [{v} + 0x44]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x44]",
        "fld dword ptr [{v} + 0x64]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x64]";
        b = base(), s = uv, v = a(VERTS)
    );
}

/// The draw (`sub_41ADE1`): eax 4 vertices, ebx `g_polygonVertices`.
pub fn draw(r: &Regs, part: Part) -> Result<(), Refused> {
    let mut d = Regs {
        eax: 4,
        ebx: a(VERTS),
        ..*r
    };
    part(DRAW, &mut d)
}

// ---- for the self-tests ----

/// The calls noted: (place, eax, ebx, ecx, esi, edi, and for a draw the vertices; for a
/// quad the two rectangles).
type Noted = (u32, [u32; 5], Vec<u8>);
static NOTED: Mutex<Vec<Noted>> = Mutex::new(Vec::new());

fn bytes(at: u32, n: usize) -> Vec<u8> {
    unsafe { std::slice::from_raw_parts(at as *const u8, n) }.to_vec()
}

/// The stand-ins' handler and the Rust side's part: note the call (and leave the
/// registers as the noted calls would, every one of them keeping them).
pub fn note(place: u32, r: &mut Regs) {
    let extra = match place {
        DRAW => bytes(a(VERTS), 0x80),
        QUAD | TURNED => [bytes(r.eax, 16), bytes(r.ebx, 16)].concat(),
        _ => Vec::new(),
    };
    NOTED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.esi, r.edi], extra));
}

fn note_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}

fn handler(place: u32, r: &mut Regs) {
    note(place, r);
}

/// A copy of one of the five whose calls (at `calls`) go to the stand-ins by place.
pub fn copy_of(
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
) -> Result<RedirectedCopy, &'static str> {
    crate::install::stand_ins::set_handler(handler);
    unsafe {
        RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|c| {
            PLACES
                .iter()
                .position(|&k| rva(k) == c)
                .map(|n| crate::install::stand_ins::stand_in(n as u32))
        })
    }
}

/// What the tests compare: the vertices, the screen's tables and layout words.
pub fn test_regions() -> Vec<Region> {
    vec![
        (a(VERTS), 0x80),
        (a(0x5183CC), 4),
        (a(0x5BEAD0), 0x140),
        (a(0x5C2A0C), 0x20),
        (a(0x5BEBF0), 0x18),
    ]
}

/// One case: the copy and the Rust version (its calls noted) from the same state.
pub fn both(
    t: &mut SelfTest,
    what: &str,
    copy: &RedirectedCopy,
    entry: Regs,
    fill: &dyn Fn(),
    rust: unsafe fn(&Regs, Part) -> Result<Regs, Refused>,
) {
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    let all = test_regions();
    fill();
    NOTED.lock().unwrap().clear();
    let mut ro = entry;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = (
        Some(regs(&ro)),
        std::mem::take(&mut *NOTED.lock().unwrap()),
        capture(&all),
    );
    fill();
    let mine = unsafe { rust(&entry, &mut note_part) };
    let mine = (
        mine.as_ref().ok().map(regs),
        std::mem::take(&mut *NOTED.lock().unwrap()),
        capture(&all),
    );
    t.check(theirs == mine, || {
        let diff = theirs.2.iter().zip(&mine.2).position(|(x, y)| x != y);
        let call = theirs.1.iter().zip(&mine.1).position(|(x, y)| x != y);
        let shown = call.map(|n| (n, &theirs.1[n], &mine.1[n]));
        format!(
            "{what}: original {:x?} ({} calls); rust {:x?} ({} calls); first different call \
             (index, original, rust) {shown:x?}; memory first differs at {diff:x?}",
            theirs.0,
            theirs.1.len(),
            mine.0,
            mine.1.len()
        )
    });
}
