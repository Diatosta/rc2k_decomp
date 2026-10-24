//! `marks::hexagons_frame` (0x4BC0B1, with its chunk 0x4BB94D .. 0x4BBE62): the hexagons'
//! frame. With the drift on (`dword_7652A0`) `marks::hexagons_drift`; `marks::hexagons_setup`;
//! then each live item (counted in `dword_814710`): with the drift on, its motion eased
//! toward the drift at its cell and its place moved; it dies off the map (x or z past
//! 0x7FF, as `(x | z << 1) & 0x800`) or on `g_marksEdgeMask`; else its sprite: the place
//! turned, unless the depth is below 4096.0 the six corners (its kind's offsets)
//! projected into the buffer at `dword_816232` (0xC0 bytes each; at `unk_628A44` back to
//! `unk_625A44`, `dword_62423C` 0x300, `dword_625A40` 0x180 and the batch drawn by
//! `sub_41ADB9`, the platform's). Then, with fewer live than the settings' +0x273C and the
//! drift on, `marks::hexagons_spawn`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BAC87` (0x4BAF08). The x87 runs are the original's instructions over the
//! items and the game's variables; its calls through `part` (1 the drift, 2 the set-up, 3
//! the batch's draw, 4 the spawn), in the game their Rust versions (the draw through the
//! game: section 15). Refuses, before anything, null settings and a live item of a kind
//! past 7. Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go
//! to stand-ins, against the Rust version, on the game's items with a few live.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::marks::expire_hexagons::{ITEM_LEN, ITEMS, ITEMS_RVA};
use crate::marks::setup::SETTINGS_RVA;
use crate::math::x87;

const DRIFTING_RVA: usize = rva(0x7652A0); // dword_7652A0
const LIVE_RVA: usize = rva(0x814710); // dword_814710
const CELL_X_RVA: usize = rva(0x71C880); // dword_71C880
const WHOLE_RVA: [usize; 2] = [rva(0x815182), rva(0x815186)]; // dword_815182 / 6
const EDGE_RVA: usize = rva(0x809BC3); // g_marksEdgeMask
const DEPTH_RVA: usize = rva(0x814CB2); // flt_814CB2
const BUFFER_RVA: usize = rva(0x816232); // dword_816232: where the next sprite goes
const BUFFER_START: usize = rva(0x625A44); // unk_625A44
const BUFFER_END: usize = rva(0x628A44); // unk_628A44
const BATCH: [(usize, u32); 2] = [(rva(0x62423C), 0x300), (rva(0x625A40), 0x180)];
const KINDS: u32 = 8;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The original's 0x4BC107 .. 0x4BC12E: the drift at the item's cell (`g_marksHexagonCurve`[k]) into `flt_814772`, `flt_814776`.
unsafe fn drift_cell(b: u32, k: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + {k} + 0x4147C2]",
            "fmul dword ptr [{b} + 0x411BE7]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x411BDF]",
            "fstp dword ptr [{b} + 0x414772]",
            "fld dword ptr [{b} + 0x411BE3]",
            "fstp dword ptr [{b} + 0x414776]",
            b = in(reg) b,
            k = in(reg) k,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BC131 .. 0x4BC17B: the item's motion (+0x1C, +0x20) eased toward the drift times its kind's `flt_814BC2`[k] by `flt_81622A`, its place moved by it.
unsafe fn drift_move(b: u32, k: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x414772]",
            "fmul dword ptr [{b} + {k} + 0x414BC2]",
            "fsub dword ptr [{s} + 0x1C]",
            "fmul dword ptr [{b} + 0x41622A]",
            "fadd dword ptr [{s} + 0x1C]",
            "fstp dword ptr [{s} + 0x1C]",
            "fld dword ptr [{b} + 0x414776]",
            "fmul dword ptr [{b} + {k} + 0x414BC2]",
            "fsub dword ptr [{s} + 0x20]",
            "fmul dword ptr [{b} + 0x41622A]",
            "fadd dword ptr [{s} + 0x20]",
            "fstp dword ptr [{s} + 0x20]",
            "fld dword ptr [{s} + 0x14]",
            "fadd dword ptr [{s} + 0x1C]",
            "fld dword ptr [{s} + 0x18]",
            "fadd dword ptr [{s} + 0x20]",
            "fstp dword ptr [{s} + 0x18]",
            "fstp dword ptr [{s} + 0x14]",
            b = in(reg) b,
            k = in(reg) k,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BC1DD .. 0x4BC1EF and 0x4BB94D .. 0x4BB9BE: the cell place scaled and turned into `flt_814CAA` .., the depth also in `flt_814734`.
unsafe fn sprite_depth(b: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{b} + 0x415182]",
            "fld dword ptr [{b} + 0x42E2A6]",
            "fisub dword ptr [{b} + 0x415186]",
            "fmul dword ptr [{b} + 0x414C0A]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x414C06]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x414C1A]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x414C26]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x414C0E]",
            "fstp dword ptr [{b} + 0x414CAA]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x414C1E]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x414C2A]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x414C12]",
            "fstp dword ptr [{b} + 0x414CAE]",
            "fmul dword ptr [{b} + 0x414C2E]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x414C22]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x414C16]",
            "fst dword ptr [{b} + 0x414734]",
            "fstp dword ptr [{b} + 0x414CB2]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BB9CE .. 0x4BBB1E: the six corners (the kind's offsets at `flt_814CFE`[c] ..) into `flt_814CB6` .., the depth's inverse kept.
unsafe fn sprite_corners(b: u32, c: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x414CB2]",
            "fdivr dword ptr [{b} + 0x224088]",
            "fld dword ptr [{b} + 0x414CAA]",
            "fadd dword ptr [{b} + {c} + 0x414CFE]",
            "fstp dword ptr [{b} + 0x414CB6]",
            "fld dword ptr [{b} + 0x414CAA]",
            "fadd dword ptr [{b} + {c} + 0x414D0A]",
            "fstp dword ptr [{b} + 0x414CC2]",
            "fld dword ptr [{b} + 0x414CAA]",
            "fadd dword ptr [{b} + {c} + 0x414D16]",
            "fstp dword ptr [{b} + 0x414CCE]",
            "fld dword ptr [{b} + 0x414CAA]",
            "fadd dword ptr [{b} + {c} + 0x414D22]",
            "fstp dword ptr [{b} + 0x414CDA]",
            "fld dword ptr [{b} + 0x414CAA]",
            "fadd dword ptr [{b} + {c} + 0x414D2E]",
            "fstp dword ptr [{b} + 0x414CE6]",
            "fld dword ptr [{b} + 0x414CAA]",
            "fadd dword ptr [{b} + {c} + 0x414D3A]",
            "fstp dword ptr [{b} + 0x414CF2]",
            "fld dword ptr [{b} + 0x414CAE]",
            "fadd dword ptr [{b} + {c} + 0x414D02]",
            "fstp dword ptr [{b} + 0x414CBA]",
            "fld dword ptr [{b} + 0x414CAE]",
            "fadd dword ptr [{b} + {c} + 0x414D0E]",
            "fstp dword ptr [{b} + 0x414CC6]",
            "fld dword ptr [{b} + 0x414CAE]",
            "fadd dword ptr [{b} + {c} + 0x414D1A]",
            "fstp dword ptr [{b} + 0x414CD2]",
            "fld dword ptr [{b} + 0x414CAE]",
            "fadd dword ptr [{b} + {c} + 0x414D26]",
            "fstp dword ptr [{b} + 0x414CDE]",
            "fld dword ptr [{b} + 0x414CAE]",
            "fadd dword ptr [{b} + {c} + 0x414D32]",
            "fstp dword ptr [{b} + 0x414CEA]",
            "fld dword ptr [{b} + 0x414CAE]",
            "fadd dword ptr [{b} + {c} + 0x414D3E]",
            "fstp dword ptr [{b} + 0x414CF6]",
            "fld dword ptr [{b} + 0x414CB2]",
            "fadd dword ptr [{b} + {c} + 0x414D06]",
            "fstp dword ptr [{b} + 0x414CBE]",
            "fld dword ptr [{b} + 0x414CB2]",
            "fadd dword ptr [{b} + {c} + 0x414D12]",
            "fstp dword ptr [{b} + 0x414CCA]",
            "fld dword ptr [{b} + 0x414CB2]",
            "fadd dword ptr [{b} + {c} + 0x414D1E]",
            "fstp dword ptr [{b} + 0x414CD6]",
            "fld dword ptr [{b} + 0x414CB2]",
            "fadd dword ptr [{b} + {c} + 0x414D2A]",
            "fstp dword ptr [{b} + 0x414CE2]",
            "fld dword ptr [{b} + 0x414CB2]",
            "fadd dword ptr [{b} + {c} + 0x414D36]",
            "fstp dword ptr [{b} + 0x414CEE]",
            "fld dword ptr [{b} + 0x414CB2]",
            "fadd dword ptr [{b} + {c} + 0x414D42]",
            "fstp dword ptr [{b} + 0x414CFA]",
            b = in(reg) b,
            c = in(reg) c,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BBB24 .. 0x4BBE28: the centre and corners projected into the buffer at e (six vertices), drawn in by `flt_81619A` / `E`, their depth.
unsafe fn sprite_project(b: u32, e: u32) {
    unsafe {
        core::arch::asm!(
            "fstp dword ptr [{b} + 0x414CB2]",
            "fld dword ptr [{b} + 0x414CAE]",
            "fmul dword ptr [{b} + 0x414CB2]",
            ".byte 0xD9, 0xE0",
            "fld dword ptr [{b} + 0x414CAA]",
            "fmul dword ptr [{b} + 0x414CB2]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x414CAA]",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x414CAE]",
            "fld dword ptr [{b} + 0x414CB6]",
            "fmul dword ptr [{b} + 0x414CB2]",
            "fld dword ptr [{b} + 0x414CBA]",
            "fmul dword ptr [{b} + 0x414CB2]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fst dword ptr [{e} + 0x1C]",
            "fstp dword ptr [{e} + 0x4]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fst dword ptr [{e} + 0x18]",
            "fstp dword ptr [{e} + 0x0]",
            "fld dword ptr [{b} + 0x414CC2]",
            "fmul dword ptr [{b} + 0x414CB2]",
            "fld dword ptr [{b} + 0x414CC6]",
            "fmul dword ptr [{b} + 0x414CB2]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fst dword ptr [{e} + 0x3C]",
            "fstp dword ptr [{e} + 0x24]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fst dword ptr [{e} + 0x38]",
            "fstp dword ptr [{e} + 0x20]",
            "fld dword ptr [{b} + 0x414CCE]",
            "fmul dword ptr [{b} + 0x414CB2]",
            "fld dword ptr [{b} + 0x414CD2]",
            "fmul dword ptr [{b} + 0x414CB2]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fst dword ptr [{e} + 0x5C]",
            "fstp dword ptr [{e} + 0x44]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fst dword ptr [{e} + 0x58]",
            "fstp dword ptr [{e} + 0x40]",
            "fld dword ptr [{b} + 0x414CDA]",
            "fmul dword ptr [{b} + 0x414CB2]",
            "fld dword ptr [{b} + 0x414CDE]",
            "fmul dword ptr [{b} + 0x414CB2]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fst dword ptr [{e} + 0x7C]",
            "fstp dword ptr [{e} + 0x64]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fst dword ptr [{e} + 0x78]",
            "fstp dword ptr [{e} + 0x60]",
            "fld dword ptr [{b} + 0x414CE6]",
            "fmul dword ptr [{b} + 0x414CB2]",
            "fld dword ptr [{b} + 0x414CEA]",
            "fmul dword ptr [{b} + 0x414CB2]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fst dword ptr [{e} + 0x9C]",
            "fstp dword ptr [{e} + 0x84]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fst dword ptr [{e} + 0x98]",
            "fstp dword ptr [{e} + 0x80]",
            "fld dword ptr [{b} + 0x414CF2]",
            "fmul dword ptr [{b} + 0x414CB2]",
            "fld dword ptr [{b} + 0x414CF6]",
            "fmul dword ptr [{b} + 0x414CB2]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fst dword ptr [{e} + 0xBC]",
            "fstp dword ptr [{e} + 0xA4]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fst dword ptr [{e} + 0xB8]",
            "fstp dword ptr [{e} + 0xA0]",
            "fld dword ptr [{b} + 0x414CAA]",
            "fmul dword ptr [{b} + 0x416192]",
            "fld dword ptr [{b} + 0x414CAE]",
            "fmul dword ptr [{b} + 0x416196]",
            ".byte 0xD9, 0xC9",
            "fstp dword ptr [{b} + 0x414C9E]",
            "fstp dword ptr [{b} + 0x414CA2]",
            "fld dword ptr [{e} + 0x18]",
            "fsub dword ptr [{b} + 0x414CAA]",
            "fmul dword ptr [{b} + 0x41619A]",
            "fadd dword ptr [{b} + 0x414C9E]",
            "fstp dword ptr [{e} + 0x18]",
            "fld dword ptr [{e} + 0x1C]",
            "fsub dword ptr [{b} + 0x414CAE]",
            "fmul dword ptr [{b} + 0x41619E]",
            "fadd dword ptr [{b} + 0x414CA2]",
            "fstp dword ptr [{e} + 0x1C]",
            "fld dword ptr [{e} + 0x38]",
            "fsub dword ptr [{b} + 0x414CAA]",
            "fmul dword ptr [{b} + 0x41619A]",
            "fadd dword ptr [{b} + 0x414C9E]",
            "fstp dword ptr [{e} + 0x38]",
            "fld dword ptr [{e} + 0x3C]",
            "fsub dword ptr [{b} + 0x414CAE]",
            "fmul dword ptr [{b} + 0x41619E]",
            "fadd dword ptr [{b} + 0x414CA2]",
            "fstp dword ptr [{e} + 0x3C]",
            "fld dword ptr [{e} + 0x58]",
            "fsub dword ptr [{b} + 0x414CAA]",
            "fmul dword ptr [{b} + 0x41619A]",
            "fadd dword ptr [{b} + 0x414C9E]",
            "fstp dword ptr [{e} + 0x58]",
            "fld dword ptr [{e} + 0x5C]",
            "fsub dword ptr [{b} + 0x414CAE]",
            "fmul dword ptr [{b} + 0x41619E]",
            "fadd dword ptr [{b} + 0x414CA2]",
            "fstp dword ptr [{e} + 0x5C]",
            "fld dword ptr [{e} + 0x78]",
            "fsub dword ptr [{b} + 0x414CAA]",
            "fmul dword ptr [{b} + 0x41619A]",
            "fadd dword ptr [{b} + 0x414C9E]",
            "fstp dword ptr [{e} + 0x78]",
            "fld dword ptr [{e} + 0x7C]",
            "fsub dword ptr [{b} + 0x414CAE]",
            "fmul dword ptr [{b} + 0x41619E]",
            "fadd dword ptr [{b} + 0x414CA2]",
            "fstp dword ptr [{e} + 0x7C]",
            "fld dword ptr [{e} + 0x98]",
            "fsub dword ptr [{b} + 0x414CAA]",
            "fmul dword ptr [{b} + 0x41619A]",
            "fadd dword ptr [{b} + 0x414C9E]",
            "fstp dword ptr [{e} + 0x98]",
            "fld dword ptr [{e} + 0x9C]",
            "fsub dword ptr [{b} + 0x414CAE]",
            "fmul dword ptr [{b} + 0x41619E]",
            "fadd dword ptr [{b} + 0x414CA2]",
            "fstp dword ptr [{e} + 0x9C]",
            "fld dword ptr [{e} + 0xB8]",
            "fsub dword ptr [{b} + 0x414CAA]",
            "fmul dword ptr [{b} + 0x41619A]",
            "fadd dword ptr [{b} + 0x414C9E]",
            "fstp dword ptr [{e} + 0xB8]",
            "fld dword ptr [{e} + 0xBC]",
            "fsub dword ptr [{b} + 0x414CAE]",
            "fmul dword ptr [{b} + 0x41619E]",
            "fadd dword ptr [{b} + 0x414CA2]",
            "fstp dword ptr [{e} + 0xBC]",
            "fld dword ptr [{b} + 0x414734]",
            "fadd dword ptr [{b} + 0x36529C]",
            "fmul dword ptr [{b} + 0x414C02]",
            "fst dword ptr [{e} + 0x8]",
            "fst dword ptr [{e} + 0x28]",
            "fst dword ptr [{e} + 0x48]",
            "fst dword ptr [{e} + 0x68]",
            "fst dword ptr [{e} + 0x88]",
            "fstp dword ptr [{e} + 0xA8]",
            b = in(reg) b,
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// Why it refuses, if it does.
unsafe fn refusal() -> Option<&'static str> {
    unsafe {
        if game_read::<u32>(SETTINGS_RVA) == 0 {
            return Some("no marks settings");
        }
        let base = game_addr(ITEMS_RVA) as u32;
        let bad = (0..ITEMS).any(|n| {
            let e = base + n * ITEM_LEN;
            get(e) == 1 && get(e + 4) >= KINDS
        });
        bad.then_some("a live hexagon of a kind past 7")
    }
}

/// The batch's draw (`d3d::draw_marks_hexagons`; its result unread).
pub fn draw_batch() -> Result<(), Refused> {
    crate::d3d::draw_marks_hexagons::draw_marks_hexagons();
    Ok(())
}

/// The Rust version (see the header).
pub unsafe fn hexagons_frame() -> Result<(), Refused> {
    if let Some(why) = unsafe { refusal() } {
        return Err(BadInput(why).into());
    }
    unsafe {
        let b = game_addr(0) as u32;
        if game_read::<u32>(DRIFTING_RVA) != 0 {
            crate::marks::hexagons_drift::hexagons_drift()?;
        }
        crate::marks::hexagons_setup::hexagons_setup();
        game_write(LIVE_RVA, 0u32);
        let base = game_addr(ITEMS_RVA) as u32;
        for n in 0..ITEMS {
            let e = base + n * ITEM_LEN;
            if get(e) != 1 {
                continue;
            }
            game_write(LIVE_RVA, game_read::<u32>(LIVE_RVA).wrapping_add(1));
            if game_read::<u32>(DRIFTING_RVA) != 0 {
                let x = x87::to_int(get(e + 0x14));
                game_write(CELL_X_RVA, x);
                drift_cell(b, 4 * (((x as u32) >> 3) & 0xFF));
                drift_move(b, 4 * get(e + 4), e);
            }
            let x = x87::to_int(get(e + 0x14));
            let z = x87::to_int(get(e + 0x18));
            game_write(WHOLE_RVA[0], x);
            game_write(WHOLE_RVA[1], z);
            if (x as u32 | (z as u32).wrapping_add(z as u32)) & 0x800 != 0 {
                put(e, 0);
                continue;
            }
            let shape = get(e + 4);
            let cell = ((((z as u32) >> 3) & 0x7F) << 8) + (((x as u32) >> 3) & 0xFF);
            if game_read::<u8>(EDGE_RVA + cell as usize) != 0 {
                put(e, 0);
                continue;
            }
            sprite_depth(b);
            if game_read::<i32>(DEPTH_RVA) < 0x4580_0000 {
                continue;
            }
            sprite_corners(b, shape.wrapping_mul(0x48));
            sprite_project(b, game_read(BUFFER_RVA));
            let next = game_read::<u32>(BUFFER_RVA).wrapping_add(0xC0);
            game_write(BUFFER_RVA, next);
            if next == game_addr(BUFFER_END) as u32 {
                game_write(BUFFER_RVA, game_addr(BUFFER_START) as u32);
                for (r, v) in BATCH {
                    game_write(r, v);
                }
                draw_batch()?;
            }
        }
        let settings: u32 = game_read(SETTINGS_RVA);
        let live: u32 = game_read(LIVE_RVA);
        if (live as i32) < get(settings + 0x273C) as i32 && game_read::<u32>(DRIFTING_RVA) != 0 {
            crate::marks::hexagons_spawn::hexagons_spawn();
        }
    }
    Ok(())
}
