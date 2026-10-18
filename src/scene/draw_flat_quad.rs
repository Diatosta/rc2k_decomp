//! `scene::draw_flat_quad` (0x44AF85): a flat quad on the screen through the polygon
//! pipeline: `g_polygonDrawn` 0; the four polygon records' vertex pointers
//! (`dword_623E40` .. 0x20 apart), texture places (`dword_623DF0` .. u, `dword_623E00` ..
//! v), the corners' x (`dword_623E10` left, `dword_623E14` right) and y (`dword_623E20`
//! top, `dword_623E24` bottom) as floats (through `dword_62400C`), depth `dword_5C5E20`;
//! the entries -1, colour `dword_623DE8`'s low 24 bits, the records' +0x18 0; four
//! vertices, the record pool, the list 0 .. 3; then `scene::scale_texture_coordinates`,
//! `scene::clip_polygon_to_screen` and, with three vertices or more left, mode 6
//! (`g_polygonMode`), `dword_6240E0` 1, `dword_6240E4` 0, `scene::depth_buffer_off`,
//! `scene::draw_polygon`, `g_polygonDrawn` 1, `scene::depth_buffer_on`; the colour back to
//! white.
//!
//! In:  nothing (`dword_623DE8` .. `dword_623E24`, set by the caller)
//! Out: eax 0, ecx 4, esi the record pool going into the calls; then as they leave them
//!
//! Callers: `sub_45B86E` (three), and code at 0x4436B2, 0x4437C1, 0x45BCD8, 0x45BDD0.
//! The calls through `part` (1 the scale, 2 the clip, 3 / 5 the depth
//! buffer off / on, 4 the draw), in the game their Rust versions (a refusal of theirs
//! comes after the writes; the original, run on it, writes them again from the same
//! inputs). Shadow mode runs the original alone (it draws). Self-test: a copy whose calls
//! go to stand-ins, against the Rust version, quads by lot.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::math::x87;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::draw_flat_quad", rva(0x44AF85), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DRAWN_RVA: usize = rva(0x624134); // g_polygonDrawn
const VERTEX_RVA: usize = rva(0x623E40); // dword_623E40: four of 0x20
const RECORDS_RVA: usize = rva(0x975510); // g_polygonEntries: records of 0x44
const UV_RVA: usize = rva(0x623DF0); // dword_623DF0 .. dword_623E0C
const CORNERS_RVA: usize = rva(0x623E10); // left, right; dword_623E20 top, bottom
const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C
const DEPTH_RVA: usize = rva(0x5C5E20); // dword_5C5E20
const COLOUR_IN_RVA: usize = rva(0x623DE8); // dword_623DE8
const COLOUR_RVA: usize = rva(0x6240CC); // g_polygonColour
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const POOL_RVA: usize = rva(0x977F10); // g_polygonRecordPool
const NEXT_RECORD_RVA: usize = rva(0x6240A4); // g_polygonNextRecord
const LIST_RVA: usize = rva(0x977710); // g_polygonList
const NEXT_ENTRY_RVA: usize = rva(0x624114); // g_polygonNextEntry
const MODE_RVA: usize = rva(0x63DAB4); // g_polygonMode
const SIDES_RVA: [usize; 2] = [rva(0x6240E0), rva(0x6240E4)];
const RECORD: usize = 0x44;

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn set(r: usize, v: u32) {
    unsafe { game_write(r, v) };
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_flat_quad(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        set(DRAWN_RVA, 0);
        for n in 0..4 {
            let rec = RECORDS_RVA + RECORD * n;
            set(rec + 4, game_addr(VERTEX_RVA + 0x20 * n) as u32);
            set(rec + 8, game_read(UV_RVA + 4 * n));
            set(rec + 0xC, game_read(UV_RVA + 0x10 + 4 * n));
        }
        let float = |r: usize| {
            let v = x87::from_int(game_read(r));
            set(SCRATCH_RVA, v);
            v
        };
        let [left, right, top, bottom] = [
            CORNERS_RVA,
            CORNERS_RVA + 4,
            CORNERS_RVA + 0x10,
            CORNERS_RVA + 0x14,
        ]
        .map(float);
        let v = |n: usize, o: usize, x: u32| set(VERTEX_RVA + 0x20 * n + o, x);
        for (n, x, y) in [
            (0, left, top),
            (1, right, top),
            (2, right, bottom),
            (3, left, bottom),
        ] {
            v(n, 0xC, x);
            v(n, 0x1C, y);
        }
        let depth: u32 = game_read(DEPTH_RVA);
        for n in 0..4 {
            v(n, 0x18, depth);
        }
        for n in 0..4 {
            set(RECORDS_RVA + RECORD * n, 0xFFFF_FFFF);
        }
        set(COLOUR_RVA, game_read::<u32>(COLOUR_IN_RVA) & 0xFF_FFFF);
        for n in 0..4 {
            set(RECORDS_RVA + RECORD * n + 0x18, 0);
        }
        set(COUNT_RVA, 4);
        g.esi = game_addr(POOL_RVA) as u32;
        set(NEXT_RECORD_RVA, g.esi);
        for n in 0..4u32 {
            set(LIST_RVA + 4 * n as usize, n);
        }
        set(LIST_RVA + 16, 0);
        set(NEXT_ENTRY_RVA, 4);
        [g.eax, g.ecx] = [0, 4];
        part(1, &mut g)?;
        part(2, &mut g)?;
        if game_read::<u32>(COUNT_RVA) >= 3 {
            set(MODE_RVA, 6);
            set(SIDES_RVA[0], 1);
            set(SIDES_RVA[1], 0);
            part(3, &mut g)?;
            part(4, &mut g)?;
            set(DRAWN_RVA, 1);
            part(5, &mut g)?;
        }
        set(COLOUR_RVA, 0xFF_FFFF);
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::scene::*;
    unsafe {
        match place {
            1 => scale_texture_coordinates::scale_texture_coordinates()?.apply(r),
            2 => clip_polygon_to_screen::clip_polygon_to_screen()?,
            3 => depth_buffer_on::depth_buffer(r, false),
            4 => draw_polygon::draw_polygon_in_game(r)?,
            _ => depth_buffer_on::depth_buffer(r, true),
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
    match unsafe { draw_flat_quad(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov g_polygonDrawn, 0
    let entry: &[u8] = &[0xC7, 0x05, 0x34, 0x41, 0x62, 0x00, 0x00, 0x00, 0x00, 0x00];
    let copy = match copy(
        0x44AF85,
        0x1ED,
        entry,
        &[0x198, 0x19D, 0x1C9, 0x1CE, 0x1DD],
        &[
            (0x44E2FB, 1),
            (0x44D15A, 2),
            (0x44E3F1, 3),
            (0x44BA2C, 4),
            (0x44E41F, 5),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![
        at(DRAWN_RVA),
        (game_addr(VERTEX_RVA) as u32, 0x80),
        (game_addr(RECORDS_RVA) as u32, RECORD * 4),
        at(SCRATCH_RVA),
        at(COLOUR_RVA),
        at(COUNT_RVA),
        at(NEXT_RECORD_RVA),
        (game_addr(LIST_RVA) as u32, 0x14),
        at(NEXT_ENTRY_RVA),
        at(MODE_RVA),
        at(SIDES_RVA[0]),
        at(SIDES_RVA[1]),
    ];
    let mut vars = regions.clone();
    vars.push((game_addr(COLOUR_IN_RVA) as u32, 0x40));
    let saved = capture(&vars);
    for case in 0..3u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                for n in 0..16u32 {
                    game_write(COLOUR_IN_RVA + 4 * n as usize, n * 0x11_0000 + case * 7 + 1);
                }
                game_write(COUNT_RVA, 9u32);
                std::ptr::write_bytes(game_addr(VERTEX_RVA) as *mut u8, 0xAA, 0x80);
            },
            &|r| unsafe { draw_flat_quad(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
