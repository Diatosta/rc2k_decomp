//! `gfx::describe_format` (0x405BB6): works a pixel format's layout out from its masks.
//!
//! In:  ebx = the format (`PixelFormat`: +0 layout, +4 four masks, +0x14 number)
//! Out: every register preserved; +0 and +0x14 written
//! If each of the four masks is empty or one run of bits (`sub_405C79`): the layout is
//! four bytes, lowest first the masks' top bits' numbers less 7, 15, 23 and 31 (an empty
//! mask counts as 0xFF; `sub_405C65`), with the sign bit set: the rotation counts the
//! colour conversions use. If the four masks together are one run as well, the format's
//! number is looked up by that layout in the table of six at `dword_4F3F78` (number,
//! layout: 1 .. 4 the 16-bit formats, 5 and 6 the 32-bit ones); else, and when it is
//! not there, the number is 0.
//! If a mask has a gap: the layout is the number of the top bit of all masks together,
//! and the number 0.
//! Three callers: `gfx::set_component_format`, the screen mode's set-up at 0x404EF2 and
//! the textures' at 0x41A1E7.
//! Refuses a null format.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::gfx::pixel_format::TEST_FORMATS;
use crate::screens::harness::Left;
use crate::service::harness::{Region, check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::describe_format", rva(0x405BB6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const NUMBERS_RVA: usize = rva(0x4F3F78); // dword_4F3F78: six of (number, layout)
const NUMBERS: usize = 6;
pub const FORMAT_LEN: usize = 0x18;

// ---- Rust version ----

/// Empty, or one run of bits.
fn one_run(mask: u32) -> bool {
    let run = mask.checked_shr(mask.trailing_zeros()).unwrap_or(0);
    run & run.wrapping_add(1) == 0
}

/// The number of the top bit, 0xFF for none.
fn top_bit(mask: u32) -> u8 {
    if mask == 0 {
        0xFF
    } else {
        31 - mask.leading_zeros() as u8
    }
}

/// The layout and the number for four masks. `numbers` is the table of (number, layout).
pub fn describe(masks: [u32; 4], numbers: &[(u32, u32)]) -> (u32, u32) {
    let all = masks.iter().fold(0, |all, &mask| all | mask);
    if !masks.iter().all(|&mask| one_run(mask)) {
        return (top_bit(all) as u32, 0);
    }
    let counts = [7u8, 15, 23, 31];
    let bytes: [u8; 4] = std::array::from_fn(|n| top_bit(masks[n]).wrapping_sub(counts[n]));
    let layout = u32::from_le_bytes(bytes) | 0x8000_0000;
    let number = one_run(all)
        .then(|| numbers.iter().find(|&&(_, known)| known == layout))
        .flatten()
        .map_or(0, |&(number, _)| number);
    (layout, number)
}

/// On the format at `format`, with the game's table.
pub unsafe fn describe_format(format: *mut u32) -> Result<(), BadInput> {
    if format.is_null() {
        return Err(BadInput("no format"));
    }
    unsafe {
        let numbers: [(u32, u32); NUMBERS] = game_read(NUMBERS_RVA);
        let masks = std::array::from_fn(|n| format.add(1 + n).read_unaligned());
        let (layout, number) = describe(masks, &numbers);
        format.write_unaligned(layout);
        format.add(5).write_unaligned(number);
    }
    Ok(())
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { describe_format(r.ebx as *mut u32) }?;
    Ok(Left::nothing())
}

fn region(r: &Regs) -> [Region; 1] {
    [(r.ebx, if r.ebx == 0 { 0 } else { FORMAT_LEN })]
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = region(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test: formats of the test's ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cases: Vec<[u32; 4]> = TEST_FORMATS.iter().map(|format| format.masks).collect();
    cases.extend([
        [0xFF, 0xFF00, 0xFF_0000, 0xFF00_0000],
        [0xFF_0000, 0xFF00, 0xFF, 0],
        [0xFF_0000, 0xFF00, 0xFF, 0xFF00_0000],
        [0x7C00, 0x03E0, 0x001F, 0x8000],
        [0xF800, 0x07E0, 0x001F, 0],
        [0x0F00, 0x00F0, 0x000F, 0xF000],
        [0, 0, 0, 0],
        [1, 0, 0, 0],
        [0x8000_0000, 0, 0, 1],
        [0x0101, 0xFF00, 0xFF_0000, 0],
        [0xFF, 0xFF00, 0xFF_0000, 0x8100_0000],
        [0xFF, 0xFF00, 0, 0xFF00_0000],
        [0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF],
        [0x5555_5555, 0, 0, 0],
    ]);
    let mut format = [0u32; 6];
    for masks in cases {
        let at = format.as_mut_ptr();
        let before = Regs {
            ebx: at as u32,
            ..Regs::SENTINEL
        };
        check_state(
            &mut t,
            &format!("masks {masks:x?}"),
            call_original,
            before,
            &|| unsafe {
                at.write(0xAAAA_AAAA);
                for (n, mask) in masks.into_iter().enumerate() {
                    at.add(1 + n).write(mask);
                }
                at.add(5).write(0xAAAA_AAAA);
            },
            &region(&before),
            &rust,
        );
    }
    t.finish()
}
